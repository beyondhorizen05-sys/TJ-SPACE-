#pragma once
#include "CoreMinimal.h"
#include "Engine/EngineTypes.h"
#include "PerformanceStreamingBudget.generated.h"

UENUM(BlueprintType) enum class ETJPerformancePriority:uint8 { Critical=0, High=1, Normal=2, Cosmetic=3 };
USTRUCT(BlueprintType) struct TJSPACE_API FTJFrameBudgetAllocation { GENERATED_BODY()
 UPROPERTY(BlueprintReadOnly) FString System; UPROPERTY(BlueprintReadOnly) ETJPerformancePriority Priority=ETJPerformancePriority::Normal; UPROPERTY(BlueprintReadOnly) float BudgetMs=0; UPROPERTY(BlueprintReadOnly) float ReportedMs=0;
};
USTRUCT(BlueprintType) struct TJSPACE_API FTJPerformanceCost { GENERATED_BODY()
 UPROPERTY(BlueprintReadOnly) FString System; UPROPERTY(BlueprintReadOnly) float Ms=0; UPROPERTY(BlueprintReadOnly) float BudgetMs=0; UPROPERTY(BlueprintReadOnly) bool bStarved=false;
};
USTRUCT(BlueprintType) struct TJSPACE_API FTJStreamingCandidate { GENERATED_BODY()
 UPROPERTY(BlueprintReadOnly) FString DistrictId; UPROPERTY(BlueprintReadOnly) float DistanceCm=0; UPROPERTY(BlueprintReadOnly) float ScreenImportance=0; UPROPERTY(BlueprintReadOnly) int32 PriorityScore=0;
};
USTRUCT(BlueprintType) struct TJSPACE_API FTJPerformanceReport { GENERATED_BODY()
 UPROPERTY(BlueprintReadOnly) float TargetFrameMs=16.6667f; UPROPERTY(BlueprintReadOnly) float TargetVRFrameMs=11.1111f; UPROPERTY(BlueprintReadOnly) float AllocatedMs=0; UPROPERTY(BlueprintReadOnly) float ReportedMs=0; UPROPERTY(BlueprintReadOnly) bool bPerformanceMode=false; UPROPERTY(BlueprintReadOnly) TArray<FTJPerformanceCost> Costs; UPROPERTY(BlueprintReadOnly) TArray<FTJFrameBudgetAllocation> Allocations;
};
UCLASS(BlueprintType) class TJSPACE_API UTJPerformanceStreamingBudget:public UObject { GENERATED_BODY()
public:
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Performance") bool AllocateFrameBudget(float BudgetMs);
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Performance") bool ReportFrameCost(const FString& System,float Ms);
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Performance") TArray<FTJStreamingCandidate> PrioritizeStreaming(const FTransform& CameraPose,float BudgetMs) const;
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Performance") bool DemoteLOD(const FString& ActorClass,int32 Level);
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Performance") bool PromoteLOD(const FString& ActorClass,int32 Level);
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Performance") bool EnterPerformanceMode();
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Performance") bool ExitPerformanceMode();
 UFUNCTION(BlueprintPure,Category="TJ SPACE|Performance") FTJPerformanceReport GetPerformanceReport() const;
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Performance") void RegisterSystem(const FString& System,ETJPerformancePriority Priority);
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Performance") void RegisterStreamingDistrict(const FString& DistrictId,const FTransform& Transform,float ScreenImportance);
private:
 void Reallocate(float BudgetMs);
 UPROPERTY() TMap<FString,FTJFrameBudgetAllocation> Allocations;
 UPROPERTY() TMap<FString,FTJPerformanceCost> Costs;
 UPROPERTY() TMap<FString,int32> LODLevels;
 UPROPERTY() TMap<FString,FTransform> DistrictTransforms;
 UPROPERTY() TMap<FString,float> DistrictImportance;
 bool bPerformanceMode=false; float TotalBudgetMs=16.6667f;
};