#pragma once
#include "CoreMinimal.h"
#include "GameFramework/SaveGame.h"
#include "SpatialWorld/SpatialWorldKernel.h"
#include "SessionPersistence.generated.h"

USTRUCT(BlueprintType)
struct TJSPACE_API FTJCameraBookmark
{
    GENERATED_BODY()
    UPROPERTY(EditAnywhere, BlueprintReadWrite) FString Name;
    UPROPERTY(EditAnywhere, BlueprintReadWrite) FTransform Transform = FTransform::Identity;
    UPROPERTY(BlueprintReadOnly) FDateTime UpdatedAt;
};

USTRUCT(BlueprintType)
struct TJSPACE_API FTJSessionState
{
    GENERATED_BODY()
    UPROPERTY(BlueprintReadOnly) FString ClientId;
    UPROPERTY(BlueprintReadOnly) TArray<FTJCameraBookmark> Bookmarks;
    UPROPERTY(BlueprintReadOnly) TArray<FString> History;
    UPROPERTY(BlueprintReadOnly) FString UIStateJson;
    UPROPERTY(BlueprintReadOnly) int32 SchemaVersion = 1;
};

UCLASS()
class TJSPACE_API UTJSessionSaveGame : public USaveGame
{
    GENERATED_BODY()
public:
    UPROPERTY() FString ClientId;
    UPROPERTY() TArray<FTJCameraBookmark> Bookmarks;
    UPROPERTY() TArray<FString> History;
    UPROPERTY() FString UIStateJson;
    UPROPERTY() int32 SchemaVersion = 1;
};

UCLASS(BlueprintType)
class TJSPACE_API UTJSessionPersistence : public UObject
{
    GENERATED_BODY()
public:
    UFUNCTION(BlueprintCallable, Category="TJ SPACE|Persistence") bool Initialize(UWorld* InWorld, UTJSpatialWorldKernel* InWorldKernel);
    UFUNCTION(BlueprintCallable, Category="TJ SPACE|Persistence") bool SaveSession(const FString& ClientId);
    UFUNCTION(BlueprintCallable, Category="TJ SPACE|Persistence") bool LoadSession(const FString& ClientId);
    UFUNCTION(BlueprintCallable, Category="TJ SPACE|Persistence") FString SnapshotWorld();
    UFUNCTION(BlueprintCallable, Category="TJ SPACE|Persistence") bool RestoreWorld(const FString& SnapshotId);
    UFUNCTION(BlueprintCallable, Category="TJ SPACE|Persistence") bool PersistBookmark(const FString& ClientId, const FString& Name, const FTransform& Transform);
    UFUNCTION(BlueprintCallable, Category="TJ SPACE|Persistence") TArray<FTJCameraBookmark> ListBookmarks(const FString& ClientId) const;
    UFUNCTION(BlueprintCallable, Category="TJ SPACE|Persistence") bool ClearSession(const FString& ClientId);
    UFUNCTION(BlueprintCallable, Category="TJ SPACE|Persistence") bool SetSessionState(const FString& ClientId, const TArray<FString>& History, const FString& UIStateJson);
    UFUNCTION(BlueprintPure, Category="TJ SPACE|Persistence") FTJSessionState GetSessionState(const FString& ClientId) const;
    UFUNCTION(BlueprintPure, Category="TJ SPACE|Persistence") bool GetBookmark(const FString& ClientId, const FString& Name, FTJCameraBookmark& OutBookmark) const;
    UFUNCTION(BlueprintPure, Category="TJ SPACE|Persistence") FString GetLastSnapshotId() const { return LastSnapshotId; }

private:
    FString SafeKey(const FString& Value) const;
    FString SessionPath(const FString& ClientId) const;
    FString SnapshotPath(const FString& SnapshotId) const;
    bool SaveSessionObject(const FTJSessionState& State) const;
    bool LoadSessionObject(const FString& ClientId, FTJSessionState& OutState) const;
    bool WriteJson(const FString& Path, const TSharedRef<FJsonObject>& Root) const;
    bool ReadJson(const FString& Path, TSharedPtr<FJsonObject>& OutRoot) const;
    TSharedRef<FJsonObject> BookmarkToJson(const FTJCameraBookmark& Bookmark) const;
    bool JsonToBookmark(const TSharedPtr<FJsonObject>& Object, FTJCameraBookmark& OutBookmark) const;
    TSharedRef<FJsonObject> TransformToJson(const FTransform& Transform) const;
    bool JsonToTransform(const TSharedPtr<FJsonObject>& Object, FTransform& OutTransform) const;

    UPROPERTY() TObjectPtr<UWorld> World;
    UPROPERTY() TObjectPtr<UTJSpatialWorldKernel> WorldKernel;
    UPROPERTY() TMap<FString, FTJSessionState> RuntimeSessions;
    FString LastSnapshotId;
};