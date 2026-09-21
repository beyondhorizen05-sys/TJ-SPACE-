#pragma once
#include "CoreMinimal.h"
#include "ErrorDegradationHandling.generated.h"

UENUM(BlueprintType) enum class ETJDegradedScope:uint8 { Backend, Sync, WorldRegion, Service, Renderer, UI, Global };
UENUM(BlueprintType) enum class ETJErrorSeverity:uint8 { Info, Warning, Error, Critical };
UENUM(BlueprintType) enum class ETJHealthState:uint8 { Healthy, Degraded, Failed, Unknown };
USTRUCT(BlueprintType) struct TJSPACE_API FTJErrorRecord { GENERATED_BODY()
 UPROPERTY(BlueprintReadOnly) FString ErrorId; UPROPERTY(BlueprintReadOnly) FString Message; UPROPERTY(BlueprintReadOnly) FString Scope; UPROPERTY(BlueprintReadOnly) ETJErrorSeverity Severity=ETJErrorSeverity::Error; UPROPERTY(BlueprintReadOnly) bool bActionable=true; UPROPERTY(BlueprintReadOnly) bool bActive=true;
};
USTRUCT(BlueprintType) struct TJSPACE_API FTJSystemHealth { GENERATED_BODY()
 UPROPERTY(BlueprintReadOnly) FString System; UPROPERTY(BlueprintReadOnly) ETJHealthState State=ETJHealthState::Unknown; UPROPERTY(BlueprintReadOnly) FString Reason; UPROPERTY(BlueprintReadOnly) int32 ActiveErrors=0;
};
USTRUCT(BlueprintType) struct TJSPACE_API FTJAggregateHealth { GENERATED_BODY()
 UPROPERTY(BlueprintReadOnly) ETJHealthState State=ETJHealthState::Unknown; UPROPERTY(BlueprintReadOnly) TArray<FTJSystemHealth> Systems; UPROPERTY(BlueprintReadOnly) TArray<FTJErrorRecord> ActiveErrors;
};
UCLASS(BlueprintType) class TJSPACE_API UTJErrorDegradationHandling:public UObject { GENERATED_BODY()
public:
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Recovery") bool HandleBackendLoss(const FString& Reason);
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Recovery") bool HandleSyncBridgeLoss();
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Recovery") bool HandleWorldLoadFailure(const FString& Region,const FString& Error);
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Recovery") bool HandleServiceCrash(const FString& PackageId,const FString& ContainerId);
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Recovery") bool EnterDegradedMode(ETJDegradedScope Scope,const FString& Reason);
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Recovery") bool ExitDegradedMode();
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Recovery") bool RecoverWorldState();
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Recovery") bool SurfaceError(const FTJErrorRecord& Error);
 UFUNCTION(BlueprintPure,Category="TJ SPACE|Recovery") FTJAggregateHealth GetSystemHealth() const;
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Recovery") bool RegisterSystem(const FString& System);
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Recovery") bool ResolveError(const FString& ErrorId);
 UFUNCTION(BlueprintPure,Category="TJ SPACE|Recovery") bool IsReadOnlyCachedMode() const{return bReadOnlyCached;}
 UFUNCTION(BlueprintPure,Category="TJ SPACE|Recovery") bool IsSyncPending() const{return bSyncPending;}
 UFUNCTION(BlueprintPure,Category="TJ SPACE|Recovery") int32 GetPendingActionCount() const{return PendingActions.Num();}
private:
 void AddError(const FString& Id,const FString& Message,ETJErrorSeverity Severity,const FString& Scope);
 void SetHealth(const FString& System,ETJHealthState State,const FString& Reason);
 UPROPERTY() TMap<FString,FTJSystemHealth> Health;
 UPROPERTY() TMap<FString,FTJErrorRecord> Errors;
 UPROPERTY() TArray<FString> PendingActions;
 bool bReadOnlyCached=false; bool bSyncPending=false; bool bDegraded=false;
};