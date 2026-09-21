#pragma once
#include "CoreMinimal.h"
#include "GameFramework/Actor.h"
#include "Components/SceneComponent.h"
#include "Components/StaticMeshComponent.h"
#include "UpdatesSigningIntegrity.generated.h"
UENUM(BlueprintType) enum class ETJUpdateState:uint8{Available,AwaitingSignature,InProgress,Complete,RolledBack,Blocked};
UENUM(BlueprintType) enum class ETJIntegrityState:uint8{Unknown,Valid,Corrupt};
USTRUCT(BlueprintType) struct TJSPACE_API FTJUpdateRecord{GENERATED_BODY() UPROPERTY(EditAnywhere,BlueprintReadWrite) FString PackageId; UPROPERTY(EditAnywhere,BlueprintReadWrite) FString Version; UPROPERTY(EditAnywhere,BlueprintReadWrite) ETJUpdateState State=ETJUpdateState::Available; UPROPERTY(EditAnywhere,BlueprintReadWrite) bool bSignatureVerified=false;};
USTRUCT(BlueprintType) struct TJSPACE_API FTJMerkleNode{GENERATED_BODY() UPROPERTY(EditAnywhere,BlueprintReadWrite) FString NodeId; UPROPERTY(EditAnywhere,BlueprintReadWrite) FString ChunkId; UPROPERTY(EditAnywhere,BlueprintReadWrite) FString Hash; UPROPERTY(EditAnywhere,BlueprintReadWrite) bool bCorrupt=false;};
UCLASS() class TJSPACE_API ATJUpdateIntegrityDistrict:public AActor{GENERATED_BODY()
public: ATJUpdateIntegrityDistrict(); bool RenderUpdateAvailable(const FString&,const FString&); bool RenderUpdateInProgress(const FString&); bool VerifyUpdateSignature(const FString&,const FString&); bool RenderRollback(const FString&,const FString&); bool RenderPackageIntegrityCheck(const FString&); bool RenderUpdateHistory(const FString&);
private: UPROPERTY() TObjectPtr<USceneComponent> Root; UPROPERTY() TObjectPtr<UStaticMeshComponent> Delivery; UPROPERTY() TObjectPtr<UStaticMeshComponent> RenovationCrew; UPROPERTY() TObjectPtr<UStaticMeshComponent> SignatureSeal; UPROPERTY() TObjectPtr<UStaticMeshComponent> RollbackRestoration; UPROPERTY() TObjectPtr<UStaticMeshComponent> MerkleTree; UPROPERTY() TObjectPtr<UStaticMeshComponent> HistoryModels; UPROPERTY() TMap<FString,FTJUpdateRecord> Updates; UPROPERTY() TMap<FString,TArray<FTJUpdateRecord>> History; void SetScalar(UPrimitiveComponent*,FName,float) const;};
UCLASS(BlueprintType) class TJSPACE_API UTJUpdatesSigningIntegrity:public UObject{GENERATED_BODY()
public: UFUNCTION(BlueprintCallable) ATJUpdateIntegrityDistrict* BuildDistrict(); UFUNCTION(BlueprintCallable) bool RenderUpdateAvailable(const FString&,const FString&); UFUNCTION(BlueprintCallable) bool RenderUpdateInProgress(const FString&); UFUNCTION(BlueprintCallable) bool VerifyUpdateSignature(const FString&,const FString&); UFUNCTION(BlueprintCallable) bool RenderRollback(const FString&,const FString&); UFUNCTION(BlueprintCallable) bool RenderPackageIntegrityCheck(const FString&); UFUNCTION(BlueprintCallable) bool RenderUpdateHistory(const FString&);
private: UPROPERTY() TObjectPtr<ATJUpdateIntegrityDistrict> District;};