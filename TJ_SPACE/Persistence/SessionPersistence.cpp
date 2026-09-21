#include "SessionPersistence.h"
#include "HAL/FileManager.h"
#include "Misc/FileHelper.h"
#include "Misc/Paths.h"
#include "Serialization/JsonSerializer.h"
#include "Serialization/JsonWriter.h"

namespace
{
    constexpr int32 SessionSchemaVersion = 1;
    constexpr int32 WorldSnapshotSchemaVersion = 1;

    static void SetVector(const TSharedRef<FJsonObject>& Object, const FString& Prefix, const FVector& Value)
    {
        Object->SetNumberField(Prefix + TEXT("X"), Value.X);
        Object->SetNumberField(Prefix + TEXT("Y"), Value.Y);
        Object->SetNumberField(Prefix + TEXT("Z"), Value.Z);
    }

    static bool GetVector(const TSharedPtr<FJsonObject>& Object, const FString& Prefix, FVector& Out)
    {
        if (!Object.IsValid() || !Object->HasField(Prefix + TEXT("X")) || !Object->HasField(Prefix + TEXT("Y")) || !Object->HasField(Prefix + TEXT("Z")))
            return false;
        Out.X = Object->GetNumberField(Prefix + TEXT("X"));
        Out.Y = Object->GetNumberField(Prefix + TEXT("Y"));
        Out.Z = Object->GetNumberField(Prefix + TEXT("Z"));
        return Out.IsFinite();
    }
}

bool UTJSessionPersistence::Initialize(UWorld* InWorld, UTJSpatialWorldKernel* InWorldKernel)
{
    if (!IsValid(InWorld) || !IsValid(InWorldKernel)) return false;
    World = InWorld;
    WorldKernel = InWorldKernel;
    return true;
}

bool UTJSessionPersistence::SaveSession(const FString& ClientId)
{
    if (ClientId.IsEmpty()) return false;
    FTJSessionState State;
    if (const FTJSessionState* Existing = RuntimeSessions.Find(ClientId)) State = *Existing;
    State.ClientId = ClientId;
    State.SchemaVersion = SessionSchemaVersion;
    return SaveSessionObject(State);
}

bool UTJSessionPersistence::LoadSession(const FString& ClientId)
{
    if (ClientId.IsEmpty()) return false;
    FTJSessionState State;
    if (!LoadSessionObject(ClientId, State)) return false;
    RuntimeSessions.Add(ClientId, State);
    return true;
}

FString UTJSessionPersistence::SnapshotWorld()
{
    if (!IsValid(World) || !IsValid(WorldKernel) || !WorldKernel->IsBooted()) return FString();

    FTJSpatialWorldLayout Layout;
    if (!WorldKernel->CaptureWorldLayout(Layout)) return FString();

    const FString SnapshotId = FString::Printf(TEXT("world-%s"), *FGuid::NewGuid().ToString(EGuidFormats::DigitsWithHyphens));
    TSharedRef<FJsonObject> Root = MakeShared<FJsonObject>();
    Root->SetNumberField(TEXT("schemaVersion"), WorldSnapshotSchemaVersion);
    Root->SetStringField(TEXT("snapshotId"), SnapshotId);
    Root->SetStringField(TEXT("kind"), TEXT("tjs.world-interface-snapshot"));
    Root->SetStringField(TEXT("worldId"), Layout.WorldId);
    Root->SetStringField(TEXT("capturedAtUtc"), FDateTime::UtcNow().ToIso8601());

    Root->SetObjectField(TEXT("streamingOrigin"), TransformToJson(FTransform(FRotator::ZeroRotator, Layout.StreamingOrigin)));

    const FTJSystemHealthState Health = WorldKernel->GetEnvironmentState();
    TSharedRef<FJsonObject> HealthJson = MakeShared<FJsonObject>();
    HealthJson->SetNumberField(TEXT("health01"), Health.Health01);
    HealthJson->SetNumberField(TEXT("load01"), Health.Load01);
    HealthJson->SetNumberField(TEXT("fault01"), Health.Fault01);
    Root->SetObjectField(TEXT("environment"), HealthJson);

    TArray<TSharedPtr<FJsonValue>> EntitiesJson;
    for (const FTJSpatialEntityState& Entity : Layout.Entities)
    {
        TSharedRef<FJsonObject> Object = MakeShared<FJsonObject>();
        Object->SetStringField(TEXT("backendId"), Entity.BackendId);
        Object->SetStringField(TEXT("districtId"), Entity.DistrictId);
        Object->SetBoolField(TEXT("registered"), Entity.bRegistered);
        Object->SetObjectField(TEXT("transform"), TransformToJson(Entity.Transform));
        EntitiesJson.Add(MakeShared<FJsonValueObject>(Object));
    }
    Root->SetArrayField(TEXT("entities"), EntitiesJson);

    TArray<TSharedPtr<FJsonValue>> DistrictsJson;
    for (const FTJSpatialDistrictState& District : Layout.Districts)
    {
        TSharedRef<FJsonObject> Object = MakeShared<FJsonObject>();
        Object->SetStringField(TEXT("id"), District.Id);
        Object->SetNumberField(TEXT("x"), District.Coord.X);
        Object->SetNumberField(TEXT("y"), District.Coord.Y);
        Object->SetObjectField(TEXT("center"), TransformToJson(FTransform(FRotator::ZeroRotator, District.Center)));
        Object->SetBoolField(TEXT("loaded"), District.bLoaded);
        DistrictsJson.Add(MakeShared<FJsonValueObject>(Object));
    }
    Root->SetArrayField(TEXT("districts"), DistrictsJson);

    if (!WriteJson(SnapshotPath(SnapshotId), Root)) return FString();
    LastSnapshotId = SnapshotId;
    return SnapshotId;
}

bool UTJSessionPersistence::RestoreWorld(const FString& SnapshotId)
{
    if (SnapshotId.IsEmpty() || !IsValid(WorldKernel) || !WorldKernel->IsBooted()) return false;

    TSharedPtr<FJsonObject> Root;
    if (!ReadJson(SnapshotPath(SnapshotId), Root) || !Root.IsValid()) return false;
    if (Root->GetIntegerField(TEXT("schemaVersion")) != WorldSnapshotSchemaVersion ||
        Root->GetStringField(TEXT("kind")) != TEXT("tjs.world-interface-snapshot")) return false;

    FTJSpatialWorldLayout Layout;
    Layout.WorldId = Root->GetStringField(TEXT("worldId"));

    const TSharedPtr<FJsonObject>* OriginObject = nullptr;
    if (!Root->TryGetObjectField(TEXT("streamingOrigin"), OriginObject) || !OriginObject)
        return false;
    FTransform OriginTransform;
    if (!JsonToTransform(*OriginObject, OriginTransform)) return false;
    Layout.StreamingOrigin = OriginTransform.GetLocation();

    const TArray<TSharedPtr<FJsonValue>>* EntityValues = nullptr;
    if (Root->TryGetArrayField(TEXT("entities"), EntityValues))
    {
        for (const TSharedPtr<FJsonValue>& Value : *EntityValues)
        {
            const TSharedPtr<FJsonObject>* Object = nullptr;
            if (!Value.IsValid() || !Value->TryGetObject(Object) || !Object || !Object->IsValid()) return false;
            FTJSpatialEntityState Entity;
            Entity.BackendId = (*Object)->GetStringField(TEXT("backendId"));
            Entity.DistrictId = (*Object)->GetStringField(TEXT("districtId"));
            Entity.bRegistered = (*Object)->GetBoolField(TEXT("registered"));
            const TSharedPtr<FJsonObject>* TransformObject = nullptr;
            if (!(*Object)->TryGetObjectField(TEXT("transform"), TransformObject) || !TransformObject || !JsonToTransform(*TransformObject, Entity.Transform)) return false;
            Layout.Entities.Add(Entity);
        }
    }

    const TArray<TSharedPtr<FJsonValue>>* DistrictValues = nullptr;
    if (Root->TryGetArrayField(TEXT("districts"), DistrictValues))
    {
        for (const TSharedPtr<FJsonValue>& Value : *DistrictValues)
        {
            const TSharedPtr<FJsonObject>* Object = nullptr;
            if (!Value.IsValid() || !Value->TryGetObject(Object) || !Object || !Object->IsValid()) return false;
            FTJSpatialDistrictState District;
            District.Id = (*Object)->GetStringField(TEXT("id"));
            District.Coord.X = (*Object)->GetIntegerField(TEXT("x"));
            District.Coord.Y = (*Object)->GetIntegerField(TEXT("y"));
            District.bLoaded = (*Object)->GetBoolField(TEXT("loaded"));
            const TSharedPtr<FJsonObject>* CenterObject = nullptr;
            if ((*Object)->TryGetObjectField(TEXT("center"), CenterObject) && CenterObject && CenterObject->IsValid())
            {
                FTransform CenterTransform;
                if (JsonToTransform(*CenterObject, CenterTransform)) District.Center = CenterTransform.GetLocation();
            }
            Layout.Districts.Add(District);
        }
    }

    const TSharedPtr<FJsonObject>* HealthObject = nullptr;
    if (Root->TryGetObjectField(TEXT("environment"), HealthObject) && HealthObject && HealthObject->IsValid())
    {
        FTJSystemHealthState Health;
        Health.Health01 = HealthObject->GetNumberField(TEXT("health01"));
        Health.Load01 = HealthObject->GetNumberField(TEXT("load01"));
        Health.Fault01 = HealthObject->GetNumberField(TEXT("fault01"));
        if (!Health.IsValid() || !WorldKernel->SetEnvironmentState(Health)) return false;
    }

    if (!WorldKernel->PersistWorldLayout(Layout)) return false;

    for (const FTJSpatialEntityState& Entity : Layout.Entities)
    {
        if (AActor* Actor = WorldKernel->ResolveBackendId(Entity.BackendId))
            Actor->SetActorTransform(Entity.Transform, false, nullptr, ETeleportType::TeleportPhysics);
    }
    LastSnapshotId = SnapshotId;
    return true;
}

bool UTJSessionPersistence::PersistBookmark(const FString& ClientId, const FString& Name, const FTransform& Transform)
{
    if (ClientId.IsEmpty() || Name.IsEmpty() || !Transform.IsValid()) return false;
    FTJSessionState& State = RuntimeSessions.FindOrAdd(ClientId);
    State.ClientId = ClientId;
    State.SchemaVersion = SessionSchemaVersion;

    FTJCameraBookmark* Existing = State.Bookmarks.FindByPredicate([&Name](const FTJCameraBookmark& B){ return B.Name == Name; });
    if (Existing)
    {
        Existing->Transform = Transform;
        Existing->UpdatedAt = FDateTime::UtcNow();
    }
    else
    {
        FTJCameraBookmark Bookmark;
        Bookmark.Name = Name;
        Bookmark.Transform = Transform;
        Bookmark.UpdatedAt = FDateTime::UtcNow();
        State.Bookmarks.Add(Bookmark);
    }
    return SaveSessionObject(State);
}

TArray<FTJCameraBookmark> UTJSessionPersistence::ListBookmarks(const FString& ClientId) const
{
    if (const FTJSessionState* State = RuntimeSessions.Find(ClientId)) return State->Bookmarks;
    FTJSessionState DiskState;
    return LoadSessionObject(ClientId, DiskState) ? DiskState.Bookmarks : TArray<FTJCameraBookmark>();
}

bool UTJSessionPersistence::ClearSession(const FString& ClientId)
{
    if (ClientId.IsEmpty()) return false;
    RuntimeSessions.Remove(ClientId);
    const FString Path = SessionPath(ClientId);
    if (!IFileManager::Get().FileExists(*Path)) return true;
    return IFileManager::Get().Delete(*Path);
}

bool UTJSessionPersistence::SetSessionState(const FString& ClientId, const TArray<FString>& History, const FString& UIStateJson)
{
    if (ClientId.IsEmpty()) return false;
    FTJSessionState& State = RuntimeSessions.FindOrAdd(ClientId);
    State.ClientId = ClientId;
    State.SchemaVersion = SessionSchemaVersion;
    State.History = History;
    State.UIStateJson = UIStateJson;
    return true;
}

FTJSessionState UTJSessionPersistence::GetSessionState(const FString& ClientId) const
{
    if (const FTJSessionState* State = RuntimeSessions.Find(ClientId)) return *State;
    FTJSessionState State;
    LoadSessionObject(ClientId, State);
    return State;
}

bool UTJSessionPersistence::GetBookmark(const FString& ClientId, const FString& Name, FTJCameraBookmark& OutBookmark) const
{
    for (const FTJCameraBookmark& Bookmark : ListBookmarks(ClientId))
    {
        if (Bookmark.Name == Name) { OutBookmark = Bookmark; return true; }
    }
    return false;
}

FString UTJSessionPersistence::SafeKey(const FString& Value) const
{
    FString Result = Value;
    for (TCHAR& Character : Result)
        if (!(FChar::IsAlnum(Character) || Character == TEXT('-') || Character == TEXT('_') || Character == TEXT('.'))) Character = TEXT('_');
    return Result.IsEmpty() ? TEXT("anonymous") : Result;
}

FString UTJSessionPersistence::SessionPath(const FString& ClientId) const
{
    return FPaths::ProjectSavedDir() / TEXT("TJ_SPACE/Sessions") / (SafeKey(ClientId) + TEXT(".sav.json"));
}

FString UTJSessionPersistence::SnapshotPath(const FString& SnapshotId) const
{
    return FPaths::ProjectSavedDir() / TEXT("TJ_SPACE/WorldSnapshots") / (SafeKey(SnapshotId) + TEXT(".json"));
}

bool UTJSessionPersistence::SaveSessionObject(const FTJSessionState& State) const
{
    TSharedRef<FJsonObject> Root = MakeShared<FJsonObject>();
    Root->SetNumberField(TEXT("schemaVersion"), SessionSchemaVersion);
    Root->SetStringField(TEXT("kind"), TEXT("tjs.operator-session"));
    Root->SetStringField(TEXT("clientId"), State.ClientId);
    Root->SetStringField(TEXT("savedAtUtc"), FDateTime::UtcNow().ToIso8601());
    Root->SetStringField(TEXT("uiStateJson"), State.UIStateJson);

    TArray<TSharedPtr<FJsonValue>> HistoryJson;
    for (const FString& Entry : State.History) HistoryJson.Add(MakeShared<FJsonValueString>(Entry));
    Root->SetArrayField(TEXT("history"), HistoryJson);

    TArray<TSharedPtr<FJsonValue>> BookmarksJson;
    for (const FTJCameraBookmark& Bookmark : State.Bookmarks) BookmarksJson.Add(MakeShared<FJsonValueObject>(BookmarkToJson(Bookmark)));
    Root->SetArrayField(TEXT("bookmarks"), BookmarksJson);

    return WriteJson(SessionPath(State.ClientId), Root);
}

bool UTJSessionPersistence::LoadSessionObject(const FString& ClientId, FTJSessionState& OutState) const
{
    TSharedPtr<FJsonObject> Root;
    if (!ReadJson(SessionPath(ClientId), Root) || !Root.IsValid()) return false;
    if (Root->GetIntegerField(TEXT("schemaVersion")) != SessionSchemaVersion ||
        Root->GetStringField(TEXT("kind")) != TEXT("tjs.operator-session") ||
        Root->GetStringField(TEXT("clientId")) != ClientId) return false;

    OutState = FTJSessionState();
    OutState.ClientId = ClientId;
    OutState.SchemaVersion = SessionSchemaVersion;
    OutState.UIStateJson = Root->GetStringField(TEXT("uiStateJson"));

    const TArray<TSharedPtr<FJsonValue>>* HistoryValues = nullptr;
    if (Root->TryGetArrayField(TEXT("history"), HistoryValues))
        for (const TSharedPtr<FJsonValue>& Value : *HistoryValues)
            if (Value.IsValid() && Value->Type == EJson::String) OutState.History.Add(Value->AsString());

    const TArray<TSharedPtr<FJsonValue>>* BookmarkValues = nullptr;
    if (Root->TryGetArrayField(TEXT("bookmarks"), BookmarkValues))
    {
        for (const TSharedPtr<FJsonValue>& Value : *BookmarkValues)
        {
            const TSharedPtr<FJsonObject>* Object = nullptr;
            if (!Value.IsValid() || !Value->TryGetObject(Object) || !Object || !Object->IsValid()) return false;
            FTJCameraBookmark Bookmark;
            if (!JsonToBookmark(*Object, Bookmark)) return false;
            OutState.Bookmarks.Add(Bookmark);
        }
    }
    return true;
}

bool UTJSessionPersistence::WriteJson(const FString& Path, const TSharedRef<FJsonObject>& Root) const
{
    IFileManager::Get().MakeDirectory(*FPaths::GetPath(Path), true);
    FString Output;
    const TSharedRef<TJsonWriter<>> Writer = TJsonWriterFactory<>::Create(&Output);
    if (!FJsonSerializer::Serialize(Root, Writer)) return false;

    const FString TempPath = Path + TEXT(".tmp");
    if (!FFileHelper::SaveStringToFile(Output, *TempPath)) return false;
    IFileManager::Get().Delete(*Path, false, true, true);
    return IFileManager::Get().Move(*Path, *TempPath, true, true);
}

bool UTJSessionPersistence::ReadJson(const FString& Path, TSharedPtr<FJsonObject>& OutRoot) const
{
    FString Input;
    if (!FFileHelper::LoadFileToString(Input, *Path)) return false;
    const TSharedRef<TJsonReader<>> Reader = TJsonReaderFactory<>::Create(Input);
    return FJsonSerializer::Deserialize(Reader, OutRoot) && OutRoot.IsValid();
}

TSharedRef<FJsonObject> UTJSessionPersistence::BookmarkToJson(const FTJCameraBookmark& Bookmark) const
{
    TSharedRef<FJsonObject> Object = MakeShared<FJsonObject>();
    Object->SetStringField(TEXT("name"), Bookmark.Name);
    Object->SetStringField(TEXT("updatedAtUtc"), Bookmark.UpdatedAt.ToIso8601());
    Object->SetObjectField(TEXT("transform"), TransformToJson(Bookmark.Transform));
    return Object;
}

bool UTJSessionPersistence::JsonToBookmark(const TSharedPtr<FJsonObject>& Object, FTJCameraBookmark& OutBookmark) const
{
    if (!Object.IsValid()) return false;
    OutBookmark.Name = Object->GetStringField(TEXT("name"));
    if (OutBookmark.Name.IsEmpty()) return false;
    FDateTime::ParseIso8601(*Object->GetStringField(TEXT("updatedAtUtc")), OutBookmark.UpdatedAt);
    const TSharedPtr<FJsonObject>* TransformObject = nullptr;
    return Object->TryGetObjectField(TEXT("transform"), TransformObject) && TransformObject && JsonToTransform(*TransformObject, OutBookmark.Transform);
}

TSharedRef<FJsonObject> UTJSessionPersistence::TransformToJson(const FTransform& Transform) const
{
    TSharedRef<FJsonObject> Object = MakeShared<FJsonObject>();
    const FVector Location = Transform.GetLocation();
    const FVector Scale = Transform.GetScale3D();
    const FQuat Rotation = Transform.GetRotation();
    SetVector(Object, TEXT("location"), Location);
    SetVector(Object, TEXT("scale"), Scale);
    Object->SetNumberField(TEXT("rotationX"), Rotation.X);
    Object->SetNumberField(TEXT("rotationY"), Rotation.Y);
    Object->SetNumberField(TEXT("rotationZ"), Rotation.Z);
    Object->SetNumberField(TEXT("rotationW"), Rotation.W);
    return Object;
}

bool UTJSessionPersistence::JsonToTransform(const TSharedPtr<FJsonObject>& Object, FTransform& OutTransform) const
{
    if (!Object.IsValid()) return false;
    FVector Location, Scale;
    if (!GetVector(Object, TEXT("location"), Location) || !GetVector(Object, TEXT("scale"), Scale)) return false;
    const FQuat Rotation(
        Object->GetNumberField(TEXT("rotationX")),
        Object->GetNumberField(TEXT("rotationY")),
        Object->GetNumberField(TEXT("rotationZ")),
        Object->GetNumberField(TEXT("rotationW")));
    if (!Rotation.IsNormalized()) return false;
    OutTransform = FTransform(Rotation, Location, Scale);
    return OutTransform.IsValid();
}