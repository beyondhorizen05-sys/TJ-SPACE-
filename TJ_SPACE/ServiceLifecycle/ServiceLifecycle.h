#pragma once
#include "CoreMinimal.h"
#include "GameFramework/Actor.h"
#include "Components/SceneComponent.h"
#include "Components/StaticMeshComponent.h"
#include "ServiceLifecycle.generated.h"

UENUM(BlueprintType) enum class ETJServiceLifecycleState:uint8{ Stopped,Starting,Running,Stopping,Error };
UENUM(BlueprintType) enum class ETJTaskSeverity:uint8{ Info,Warning,Critical };
USTRUCT(BlueprintType) struct TJSPACE_API FTJSDKAction{GENERATED_BODY() UPROPERTY(EditAnywhere,BlueprintReadWrite) FString ActionId; UPROPERTY(EditAnywhere,BlueprintReadWrite) FString DisplayName; UPROPERTY(EditAnywhere,BlueprintReadWrite) FString InputSchema;};
USTRUCT(BlueprintType) struct TJSPACE_API FTJServiceTask{GENERATED_BODY() UPROPERTY(EditAnywhere,BlueprintReadWrite) FString TaskId; UPROPERTY(EditAnywhere,BlueprintReadWrite) FString Message; UPROPERTY(EditAnywhere,BlueprintReadWrite) ETJTaskSeverity Severity=ETJTaskSeverity::Info; UPROPERTY(EditAnywhere,BlueprintReadWrite) bool bResolved=false;};
USTRUCT(BlueprintType) struct TJSPACE_API FTJHealthCheck{GENERATED_BODY() UPROPERTY(EditAnywhere,BlueprintReadWrite) FString CheckId; UPROPERTY(EditAnywhere,BlueprintReadWrite) FString Name; UPROPERTY(EditAnywhere,BlueprintReadWrite) bool bHealthy=false; UPROPERTY(EditAnywhere,BlueprintReadWrite) float Value01=0;};
USTRUCT(BlueprintType) struct TJSPACE_API FTJServiceLogEntry{GENERATED_BODY() UPROPERTY(EditAnywhere,BlueprintReadWrite) FString Timestamp; UPROPERTY(EditAnywhere,BlueprintReadWrite) FString Level; UPROPERTY(EditAnywhere,BlueprintReadWrite) FString Message;};
USTRUCT(BlueprintType) struct TJSPACE_API FTJServiceConfig{GENERATED_BODY() UPROPERTY(EditAnywhere,BlueprintReadWrite) TMap<FString,FString> Values;};
USTRUCT(BlueprintType) struct TJSPACE_API FTJServiceRuntime{GENERATED_BODY() UPROPERTY(EditAnywhere,BlueprintReadWrite) FString PackageId; UPROPERTY(EditAnywhere,BlueprintReadWrite) ETJServiceLifecycleState State=ETJServiceLifecycleState::Stopped; UPROPERTY(EditAnywhere,BlueprintReadWrite) float ForcedStopHoldSeconds=0; UPROPERTY(EditAnywhere,BlueprintReadWrite) bool bCriticalDoorBlocked=false;};

UCLASS() class TJSPACE_API ATJServiceLifecycleSpace:public AActor{
GENERATED_BODY()
public:
ATJServiceLifecycleSpace();
virtual void Tick(float DeltaSeconds) override;
bool Start(const FString& PackageId); bool Stop(const FString& PackageId,bool bGraceful);
bool RenderActions(const FString& PackageId); bool Execute(const FString& PackageId,const FString& ActionId,const FString& Input);
bool RenderTasks(const FString& PackageId); bool RenderHealth(const FString& PackageId); bool RenderLogs(const FString& PackageId); bool RenderConfig(const FString& PackageId);
bool HoldForcedStopLever(const FString& PackageId,float DeltaSeconds); bool ReleaseForcedStopLever(const FString& PackageId);
void RegisterService(const FTJServiceRuntime& Runtime,const TArray<FTJSDKAction>& Actions,const TArray<FTJServiceTask>& Tasks,const TArray<FTJHealthCheck>& Health,const TArray<FTJServiceLogEntry>& Logs,const FTJServiceConfig& Config);
bool ResolveTask(const FString& PackageId,const FString& TaskId);
private:
UPROPERTY() TObjectPtr<USceneComponent> Root; UPROPERTY() TObjectPtr<UStaticMeshComponent> BuildingConsole; UPROPERTY() TObjectPtr<UStaticMeshComponent> Ignition; UPROPERTY() TObjectPtr<UStaticMeshComponent> ShutdownConsole; UPROPERTY() TObjectPtr<UStaticMeshComponent> ForcedStopLever; UPROPERTY() TObjectPtr<UStaticMeshComponent> DoorBarrier;
TMap<FString,FTJServiceRuntime> Services; TMap<FString,TArray<FTJSDKAction>> Actions; TMap<FString,TArray<FTJServiceTask>> Tasks; TMap<FString,TArray<FTJHealthCheck>> Health; TMap<FString,TArray<FTJServiceLogEntry>> Logs; TMap<FString,FTJServiceConfig> Configs;
void SetScalar(UPrimitiveComponent* C,FName P,float V) const; void RefreshDoor(const FString& PackageId);
};

UCLASS(BlueprintType) class TJSPACE_API UTJServiceLifecycle:public UObject{
GENERATED_BODY()
public:
UFUNCTION(BlueprintCallable) ATJServiceLifecycleSpace* BuildSpace();
UFUNCTION(BlueprintCallable) bool StartService(const FString& PackageId); UFUNCTION(BlueprintCallable) bool StopService(const FString& PackageId,bool Graceful);
UFUNCTION(BlueprintCallable) bool RenderSDKActions(const FString& PackageId); UFUNCTION(BlueprintCallable) bool ExecuteAction(const FString& PackageId,const FString& ActionId,const FString& Input);
UFUNCTION(BlueprintCallable) bool RenderTasks(const FString& PackageId); UFUNCTION(BlueprintCallable) bool RenderHealthChecks(const FString& PackageId); UFUNCTION(BlueprintCallable) bool RenderServiceLogs(const FString& PackageId); UFUNCTION(BlueprintCallable) bool RenderConfiguration(const FString& PackageId);
UFUNCTION(BlueprintCallable) bool HoldForcedStopLever(const FString& PackageId,float DeltaSeconds); UFUNCTION(BlueprintCallable) bool ReleaseForcedStopLever(const FString& PackageId);
UFUNCTION(BlueprintCallable) bool ResolveTask(const FString& PackageId,const FString& TaskId);
void RegisterService(const FTJServiceRuntime& Runtime,const TArray<FTJSDKAction>& A,const TArray<FTJServiceTask>& T,const TArray<FTJHealthCheck>& H,const TArray<FTJServiceLogEntry>& L,const FTJServiceConfig& C);
private: UPROPERTY() TObjectPtr<ATJServiceLifecycleSpace> Space;
};