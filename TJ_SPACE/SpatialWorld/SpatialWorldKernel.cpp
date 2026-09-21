#include "SpatialWorldKernel.h"

#include "Components/PrimitiveComponent.h"
#include "Materials/MaterialInstanceDynamic.h"
#include "Engine/World.h"
#include "GameFramework/WorldSettings.h"
#include "HAL/FileManager.h"
#include "Misc/Paths.h"
#include "Misc/FileHelper.h"
#include "Serialization/JsonSerializer.h"
#include "Serialization/JsonWriter.h"
#include "UObject/SoftObjectPath.h"
#include "WorldPartition/WorldPartition.h"

namespace
{
    constexpr float HealthParameterDefault = 1.0f;
}

bool UTJSpatialWorldKernel::BootCitadel(const FTJSpatialServerConfig& InServerConfig)
{
    if (!InServerConfig.IsValid())
    {
        return false;
    }

    UWorld* ContextWorld = GetWorld();
    if (!IsValid(ContextWorld))
    {
        return false;
    }

    if (UWorldPartition* WorldPartition = ContextWorld->GetWorldPartition())
    {
        if (InServerConfig.bEnableStreaming && !WorldPartition->CanStream())
        {
            return false;
        }
    }
    else if (InServerConfig.bEnableStreaming)
    {
        return false;
    }

    ServerConfig = InServerConfig;
    World = ContextWorld;
    StreamingOrigin = InServerConfig.InitialOrigin;
    PersistencePath = FPaths::ProjectSavedDir() / TEXT("TJ_SPACE/CitadelLayout.json");

    bBooted = true;

    const FTJSpatialDistrictCoord OriginDistrict = WorldToDistrict(StreamingOrigin);
    FTJSpatialDistrictState OriginState;
    OriginState.Id = MakeDistrictId(OriginDistrict);
    OriginState.Coord = OriginDistrict;
    OriginState.Center = FVector(
        OriginDistrict.X * ServerConfig.DistrictCellSize + ServerConfig.DistrictCellSize * 0.5,
        OriginDistrict.Y * ServerConfig.DistrictCellSize + ServerConfig.DistrictCellSize * 0.5,
        0.0);
    OriginState.bLoaded = true;
    Districts.FindOrAdd(OriginDistrict) = OriginState;

    LoadPersistedLayout();
    return StreamDistrict(OriginDistrict);
}

bool UTJSpatialWorldKernel::RegisterSpatialEntity(const FString& BackendId, AActor* Actor)
{
    if (!bBooted || BackendId.IsEmpty() || !IsValid(Actor))
    {
        return false;
    }

    if (BackendToActor.Contains(BackendId))
    {
        return false;
    }

    if (const FString* ExistingBackendId = ActorToBackend.Find(Actor))
    {
        if (!ExistingBackendId->IsEmpty())
        {
            return false;
        }
    }

    FTJSpatialEntityState State;
    State.BackendId = BackendId;
    State.Transform = Actor->GetActorTransform();
    State.DistrictId = MakeDistrictId(WorldToDistrict(State.Transform.GetLocation()));
    State.bRegistered = true;

    BackendToActor.Add(BackendId, Actor);
    ActorToBackend.Add(Actor, BackendId);
    EntityStates.Add(BackendId, State);

    OnSpatialEntityChanged.Broadcast(State);
    return true;
}

bool UTJSpatialWorldKernel::StreamDistrict(const FTJSpatialDistrictCoord& Coord)
{
    if (!bBooted || !FMath::IsFinite(ServerConfig.DistrictCellSize) || ServerConfig.DistrictCellSize <= 0.0)
    {
        return false;
    }

    StreamingOrigin = FVector(
        Coord.X * ServerConfig.DistrictCellSize + ServerConfig.DistrictCellSize * 0.5,
        Coord.Y * ServerConfig.DistrictCellSize + ServerConfig.DistrictCellSize * 0.5,
        StreamingOrigin.Z);

    const double RangeSquared = FMath::Square(ServerConfig.DistrictLoadingRange);

    for (auto& Pair : Districts)
    {
        FTJSpatialDistrictState& District = Pair.Value;
        const double DistanceSquared = FVector::DistSquared(
            FVector(District.Center.X, District.Center.Y, StreamingOrigin.Z),
            StreamingOrigin);
        const bool bShouldLoad = DistanceSquared <= RangeSquared;

        if (District.bLoaded != bShouldLoad)
        {
            District.bLoaded = bShouldLoad;
            OnSpatialDistrictChanged.Broadcast(District);
        }
    }

    if (!Districts.Contains(Coord))
    {
        FTJSpatialDistrictState NewDistrict;
        NewDistrict.Id = MakeDistrictId(Coord);
        NewDistrict.Coord = Coord;
        NewDistrict.Center = FVector(
            Coord.X * ServerConfig.DistrictCellSize + ServerConfig.DistrictCellSize * 0.5,
            Coord.Y * ServerConfig.DistrictCellSize + ServerConfig.DistrictCellSize * 0.5,
            0.0);
        NewDistrict.bLoaded = true;
        Districts.Add(Coord, NewDistrict);
        OnSpatialDistrictChanged.Broadcast(NewDistrict);
    }
    else
    {
        FTJSpatialDistrictState& Requested = Districts.FindChecked(Coord);
        if (!Requested.bLoaded)
        {
            Requested.bLoaded = true;
            OnSpatialDistrictChanged.Broadcast(Requested);
        }
    }

    return true;
}

bool UTJSpatialWorldKernel::SetEnvironmentState(const FTJSystemHealthState& SystemHealth)
{
    if (!bBooted || !SystemHealth.IsValid())
    {
        return false;
    }

    EnvironmentState = SystemHealth;
    EnvironmentState.Health01 = FMath::Clamp(EnvironmentState.Health01, 0.0, 1.0);
    EnvironmentState.Load01 = FMath::Clamp(EnvironmentState.Load01, 0.0, 1.0);
    EnvironmentState.Fault01 = FMath::Clamp(EnvironmentState.Fault01, 0.0, 1.0);

    for (const auto& Pair : BackendToActor)
    {
        ApplyHealthToActor(Pair.Value, EnvironmentState);
    }

    OnEnvironmentStateChanged.Broadcast(EnvironmentState);
    return true;
}

bool UTJSpatialWorldKernel::PersistWorldLayout(const FTJSpatialWorldLayout& Layout)
{
    if (!bBooted || Layout.WorldId.IsEmpty())
    {
        return false;
    }

    FTJSpatialWorldLayout Snapshot = Layout;
    Snapshot.WorldId = ServerConfig.WorldId;
    Snapshot.StreamingOrigin = StreamingOrigin;

    Snapshot.Entities.Reset();
    for (const auto& Pair : EntityStates)
    {
        if (AActor* Actor = BackendToActor.FindRef(Pair.Key))
        {
            FTJSpatialEntityState State = Pair.Value;
            State.Transform = Actor->GetActorTransform();
            State.DistrictId = MakeDistrictId(WorldToDistrict(State.Transform.GetLocation()));
            Snapshot.Entities.Add(State);
        }
    }

    Snapshot.Districts.Reset();
    for (const auto& Pair : Districts)
    {
        Snapshot.Districts.Add(Pair.Value);
    }

    TSharedRef<FJsonObject> Root = MakeShared<FJsonObject>();
    Root->SetStringField(TEXT("worldId"), Snapshot.WorldId);
    Root->SetNumberField(TEXT("originX"), Snapshot.StreamingOrigin.X);
    Root->SetNumberField(TEXT("originY"), Snapshot.StreamingOrigin.Y);
    Root->SetNumberField(TEXT("originZ"), Snapshot.StreamingOrigin.Z);

    TArray<TSharedPtr<FJsonValue>> EntitiesJson;
    for (const FTJSpatialEntityState& Entity : Snapshot.Entities)
    {
        TSharedRef<FJsonObject> Object = MakeShared<FJsonObject>();
        Object->SetStringField(TEXT("backendId"), Entity.BackendId);
        Object->SetStringField(TEXT("districtId"), Entity.DistrictId);
        Object->SetBoolField(TEXT("registered"), Entity.bRegistered);

        const FVector Location = Entity.Transform.GetLocation();
        Object->SetNumberField(TEXT("x"), Location.X);
        Object->SetNumberField(TEXT("y"), Location.Y);
        Object->SetNumberField(TEXT("z"), Location.Z);

        EntitiesJson.Add(MakeShared<FJsonValueObject>(Object));
    }
    Root->SetArrayField(TEXT("entities"), EntitiesJson);

    TArray<TSharedPtr<FJsonValue>> DistrictsJson;
    for (const FTJSpatialDistrictState& District : Snapshot.Districts)
    {
        TSharedRef<FJsonObject> Object = MakeShared<FJsonObject>();
        Object->SetStringField(TEXT("id"), District.Id);
        Object->SetNumberField(TEXT("x"), District.Coord.X);
        Object->SetNumberField(TEXT("y"), District.Coord.Y);
        Object->SetBoolField(TEXT("loaded"), District.bLoaded);
        DistrictsJson.Add(MakeShared<FJsonValueObject>(Object));
    }
    Root->SetArrayField(TEXT("districts"), DistrictsJson);

    IFileManager::Get().MakeDirectory(*FPaths::GetPath(PersistencePath), true);

    FString Output;
    const TSharedRef<TJsonWriter<>> Writer = TJsonWriterFactory<>::Create(&Output);
    if (!FJsonSerializer::Serialize(Root, Writer))
    {
        return false;
    }

    return FFileHelper::SaveStringToFile(Output, *PersistencePath);
}

FTJSpatialQueryResult UTJSpatialWorldKernel::QuerySpatialState(const FTJSpatialQueryFilter& Filter) const
{
    FTJSpatialQueryResult Result;

    for (const auto& Pair : EntityStates)
    {
        const FTJSpatialEntityState& State = Pair.Value;

        if (!Filter.BackendIdPrefix.IsEmpty() && !State.BackendId.StartsWith(Filter.BackendIdPrefix))
        {
            continue;
        }

        if (!Filter.DistrictId.IsEmpty() && State.DistrictId != Filter.DistrictId)
        {
            continue;
        }

        if (Filter.bUseBounds && !PointInsideFilter(State.Transform.GetLocation(), Filter))
        {
            continue;
        }

        Result.Entities.Add(State);
    }

    Result.MatchCount = Result.Entities.Num();
    return Result;
}

bool UTJSpatialWorldKernel::ResetCitadel(bool bPreserveData)
{
    if (!bBooted)
    {
        return false;
    }

    if (bPreserveData)
    {
        FTJSpatialWorldLayout Layout;
        Layout.WorldId = ServerConfig.WorldId;
        if (!PersistWorldLayout(Layout))
        {
            return false;
        }
    }
    else if (IFileManager::Get().FileExists(*PersistencePath))
    {
        IFileManager::Get().Delete(*PersistencePath);
    }

    ClearRegistry();
    Districts.Reset();
    EnvironmentState = FTJSystemHealthState();
    StreamingOrigin = ServerConfig.InitialOrigin;

    return StreamDistrict(WorldToDistrict(StreamingOrigin));
}

AActor* UTJSpatialWorldKernel::ResolveBackendId(const FString& BackendId) const
{
    if (const TObjectPtr<AActor>* Actor = BackendToActor.Find(BackendId))
    {
        return Actor->Get();
    }
    return nullptr;
}

FString UTJSpatialWorldKernel::ResolveActorBackendId(AActor* Actor) const
{
    if (!IsValid(Actor))
    {
        return FString();
    }

    if (const FString* BackendId = ActorToBackend.Find(Actor))
    {
        return *BackendId;
    }

    return FString();
}

FString UTJSpatialWorldKernel::MakeDistrictId(const FTJSpatialDistrictCoord& Coord) const
{
    return FString::Printf(TEXT("district_%d_%d"), Coord.X, Coord.Y);
}

FTJSpatialDistrictCoord UTJSpatialWorldKernel::WorldToDistrict(const FVector& Location) const
{
    FTJSpatialDistrictCoord Result;
    Result.X = FMath::FloorToInt(Location.X / ServerConfig.DistrictCellSize);
    Result.Y = FMath::FloorToInt(Location.Y / ServerConfig.DistrictCellSize);
    return Result;
}

bool UTJSpatialWorldKernel::PointInsideFilter(const FVector& Point, const FTJSpatialQueryFilter& Filter) const
{
    const FVector Delta = Point - Filter.Center;
    return FMath::Abs(Delta.X) <= Filter.Extent.X &&
           FMath::Abs(Delta.Y) <= Filter.Extent.Y &&
           FMath::Abs(Delta.Z) <= Filter.Extent.Z;
}

bool UTJSpatialWorldKernel::ApplyHealthToActor(AActor* Actor, const FTJSystemHealthState& Health) const
{
    if (!IsValid(Actor))
    {
        return false;
    }

    bool bApplied = false;
    TArray<UPrimitiveComponent*> Primitives;
    Actor->GetComponents<UPrimitiveComponent>(Primitives);

    const float HealthValue = static_cast<float>(Health.Health01);
    const float StressValue = 1.0f - HealthValue;

    for (UPrimitiveComponent* Primitive : Primitives)
    {
        if (!IsValid(Primitive))
        {
            continue;
        }

        const int32 MaterialCount = Primitive->GetNumMaterials();
        for (int32 Index = 0; Index < MaterialCount; ++Index)
        {
            if (UMaterialInstanceDynamic* MID = Primitive->CreateAndSetMaterialInstanceDynamic(Index))
            {
                MID->SetScalarParameterValue(TEXT("DegradedStrained"), StressValue);
                MID->SetScalarParameterValue(TEXT("SystemHealth01"), HealthValue);
                MID->SetScalarParameterValue(TEXT("SystemLoad01"), static_cast<float>(Health.Load01));
                MID->SetScalarParameterValue(TEXT("SystemFault01"), static_cast<float>(Health.Fault01));
                bApplied = true;
            }
        }
    }

    return bApplied;
}

bool UTJSpatialWorldKernel::LoadPersistedLayout()
{
    if (!IFileManager::Get().FileExists(*PersistencePath))
    {
        return true;
    }

    FString Input;
    if (!FFileHelper::LoadFileToString(Input, *PersistencePath))
    {
        return false;
    }

    TSharedPtr<FJsonObject> Root;
    const TSharedRef<TJsonReader<>> Reader = TJsonReaderFactory<>::Create(Input);
    if (!FJsonSerializer::Deserialize(Reader, Root) || !Root.IsValid())
    {
        return false;
    }

    if (Root->GetStringField(TEXT("worldId")) != ServerConfig.WorldId)
    {
        return false;
    }

    StreamingOrigin.X = Root->GetNumberField(TEXT("originX"));
    StreamingOrigin.Y = Root->GetNumberField(TEXT("originY"));
    StreamingOrigin.Z = Root->GetNumberField(TEXT("originZ"));

    const TArray<TSharedPtr<FJsonValue>>* DistrictValues = nullptr;
    if (Root->TryGetArrayField(TEXT("districts"), DistrictValues))
    {
        for (const TSharedPtr<FJsonValue>& Value : *DistrictValues)
        {
            const TSharedPtr<FJsonObject>* Object = nullptr;
            if (!Value.IsValid() || !Value->TryGetObject(Object) || !Object || !Object->IsValid())
            {
                continue;
            }

            FTJSpatialDistrictState District;
            District.Id = (*Object)->GetStringField(TEXT("id"));
            District.Coord.X = (*Object)->GetIntegerField(TEXT("x"));
            District.Coord.Y = (*Object)->GetIntegerField(TEXT("y"));
            District.Center = FVector(
                District.Coord.X * ServerConfig.DistrictCellSize + ServerConfig.DistrictCellSize * 0.5,
                District.Coord.Y * ServerConfig.DistrictCellSize + ServerConfig.DistrictCellSize * 0.5,
                0.0);
            District.bLoaded = (*Object)->GetBoolField(TEXT("loaded"));
            Districts.Add(District.Coord, District);
        }
    }

    return true;
}

bool UTJSpatialWorldKernel::SavePersistedLayout() const
{
    FTJSpatialWorldLayout Layout;
    Layout.WorldId = ServerConfig.WorldId;
    return const_cast<UTJSpatialWorldKernel*>(this)->PersistWorldLayout(Layout);
}

void UTJSpatialWorldKernel::ClearRegistry()
{
    BackendToActor.Reset();
    ActorToBackend.Reset();
    EntityStates.Reset();
}
