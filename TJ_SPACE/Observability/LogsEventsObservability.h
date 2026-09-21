#pragma once
#include "CoreMinimal.h"
#include "GameFramework/Actor.h"
#include "Components/SceneComponent.h"
#include "Components/StaticMeshComponent.h"
#include "LogsEventsObservability.generated.h"

UENUM(BlueprintType) enum class ETJLogFormat:uint8{ JSON, NDJSON, CSV };
UENUM(BlueprintType) enum class ETJRetentionAction:uint8{ Keep, Clear };
USTRUCT(BlueprintType) struct TJSPACE_API FTJLogRecord{GENERATED_BODY()
 UPROPERTY(EditAnywhere,BlueprintReadWrite) double TimestampSeconds=0; UPROPERTY(EditAnywhere,BlueprintReadWrite) FString Source; UPROPERTY(EditAnywhere,BlueprintReadWrite) FString Topic; UPROPERTY(EditAnywhere,BlueprintReadWrite) FString Message;};
USTRUCT(BlueprintType) struct TJSPACE_API FTJArchiveFilter{GENERATED_BODY()
 UPROPERTY(EditAnywhere,BlueprintReadWrite) FString Source; UPROPERTY(EditAnywhere,BlueprintReadWrite) FString TopicPrefix; UPROPERTY(EditAnywhere,BlueprintReadWrite) FString Contains;};
USTRUCT(BlueprintType) struct TJSPACE_API FTJRetentionPolicy{GENERATED_BODY()
 UPROPERTY(EditAnywhere,BlueprintReadWrite) double MaxAgeSeconds=604800; UPROPERTY(EditAnywhere,BlueprintReadWrite) ETJRetentionAction Action=ETJRetentionAction::Clear;};
USTRUCT(BlueprintType) struct TJSPACE_API FTJExportRange{GENERATED_BODY()
 UPROPERTY(EditAnywhere,BlueprintReadWrite) double StartSeconds=0; UPROPERTY(EditAnywhere,BlueprintReadWrite) double EndSeconds=0;};
UCLASS() class TJSPACE_API ATJArchiveObservability:public AActor{GENERATED_BODY()
public: ATJArchiveObservability(); virtual void Tick(float DeltaSeconds) override;
bool BuildArchiveTower(); bool StreamLogsToArchive(const FString&Source); TArray<FTJLogRecord> QueryArchive(const FTJArchiveFilter&Filter) const; bool RenderEventBus(); bool RenderAuditTrail(const FString&ActorId); bool ExportLogs(ETJLogFormat Format,const FTJExportRange&Range); bool ConfigureLogRetention(const FTJRetentionPolicy&Policy);
void AddLog(const FTJLogRecord&Record); void SetListenerInsideArchive(bool bInside);
private: UPROPERTY() TObjectPtr<USceneComponent> Root; UPROPERTY() TObjectPtr<UStaticMeshComponent> ArchiveTower; UPROPERTY() TObjectPtr<UStaticMeshComponent> EventRailway; UPROPERTY() TObjectPtr<UStaticMeshComponent> Ledger; UPROPERTY() TObjectPtr<UStaticMeshComponent> ExportCrystals; UPROPERTY() TObjectPtr<UStaticMeshComponent> RetentionShelves; UPROPERTY() TArray<TObjectPtr<UStaticMeshComponent>> LogStreams; UPROPERTY() TArray<TObjectPtr<UStaticMeshComponent>> AuditRibbons; UPROPERTY() TArray<TObjectPtr<UStaticMeshComponent>> DerailedCars; UPROPERTY() TArray<FTJLogRecord> Logs; FTJRetentionPolicy Retention; double RuntimeSeconds=0; bool bArchiveAudioProfile=false;
void SetScalar(UPrimitiveComponent*C,FName P,float V) const; bool ValidTopic(const FString&Topic) const; void ApplyAudioProfile();};
UCLASS(BlueprintType) class TJSPACE_API UTTJLogsEventsObservability:public UObject{GENERATED_BODY()
public: UFUNCTION(BlueprintCallable) ATJArchiveObservability* BuildArchiveTower(); UFUNCTION(BlueprintCallable) bool StreamLogsToArchive(const FString&Source); UFUNCTION(BlueprintCallable) bool QueryArchive(const FTJArchiveFilter&Filter); UFUNCTION(BlueprintCallable) bool RenderEventBus(); UFUNCTION(BlueprintCallable) bool RenderAuditTrail(const FString&ActorId); UFUNCTION(BlueprintCallable) bool ExportLogs(ETJLogFormat Format,const FTJExportRange&Range); UFUNCTION(BlueprintCallable) bool ConfigureLogRetention(const FTJRetentionPolicy&Policy);
private: UPROPERTY() TObjectPtr<ATJArchiveObservability> Archive;};