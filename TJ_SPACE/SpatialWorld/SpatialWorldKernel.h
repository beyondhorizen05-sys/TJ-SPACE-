#pragma once

#include "CoreMinimal.h"
#include "GameFramework/Actor.h"
#include "UObject/Object.h"
#include "SpatialWorldKernel.generated.h"

USTRUCT(BlueprintType)
struct TJSPACE_API FTJSpatialServerConfig
{
    GENERATED_BODY()

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Citadel")
    FString WorldId = TEXT("citadel");

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Citadel")
    FVector InitialOrigin = FVector::ZeroVector;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Citadel")
    double DistrictCellSize = 25600.0;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Citadel")
    double DistrictLoadingRange = 76800.0;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Citadel")
    bool bEnableStreaming = true;

    bool IsValid() const
    {
        return !WorldId.IsEmpty() &&
               InitialOrigin.IsFinite() &&
               FMath::IsFinite(DistrictCellSize) && DistrictCellSize > 0.0 &&
               FMath::IsFinite(DistrictLoadingRange) && DistrictLoadingRange > 0.0;
    }
};

USTRUCT(BlueprintType)
struct TJSPACE_API FTJSpatialDistrictCoord
{
    GENERATED_BODY()

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Citadel")
    int32 X = 0;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Citadel")
    int32 Y = 0;

    bool operator==(const FTJSpatialDistrictCoord& Other) const
    {
        return X == Other.X && Y == Other.Y;
    }
};

FORCEINLINE uint32 GetTypeHash(const FTJSpatialDistrictCoord& Coord)
{
    return HashCombine(::GetTypeHash(Coord.X), ::GetTypeHash(Coord.Y));
}

USTRUCT(BlueprintType)
struct TJSPACE_API FTJSpatialEntityState
{
    GENERATED_BODY()

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Registry")
    FString BackendId;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Registry")
    FTransform Transform = FTransform::Identity;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Registry")
    FString DistrictId;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Registry")
    bool bRegistered = false;
};

USTRUCT(BlueprintType)
struct TJSPACE_API FTJSpatialDistrictState
{
    GENERATED_BODY()

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Citadel")
    FString Id;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Citadel")
    FTJSpatialDistrictCoord Coord;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Citadel")
    FVector Center = FVector::ZeroVector;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Citadel")
    bool bLoaded = false;
};

USTRUCT(BlueprintType)
struct TJSPACE_API FTJSpatialWorldLayout
{
    GENERATED_BODY()

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Persistence")
    FString WorldId;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Persistence")
    FVector StreamingOrigin = FVector::ZeroVector;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Persistence")
    TArray<FTJSpatialEntityState> Entities;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Persistence")
    TArray<FTJSpatialDistrictState> Districts;
};

USTRUCT(BlueprintType)
struct TJSPACE_API FTJSystemHealthState
{
    GENERATED_BODY()

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Environment")
    double Health01 = 1.0;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Environment")
    double Load01 = 0.0;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Environment")
    double Fault01 = 0.0;

    bool IsValid() const
    {
        return FMath::IsFinite(Health01) && FMath::IsFinite(Load01) && FMath::IsFinite(Fault01);
    }
};

USTRUCT(BlueprintType)
struct TJSPACE_API FTJSpatialQueryFilter
{
    GENERATED_BODY()

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Query")
    FString BackendIdPrefix;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Query")
    FString DistrictId;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Query")
    FVector Center = FVector::ZeroVector;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Query")
    FVector Extent = FVector::ZeroVector;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Query")
    bool bUseBounds = false;
};

USTRUCT(BlueprintType)
struct TJSPACE_API FTJSpatialQueryResult
{
    GENERATED_BODY()

    UPROPERTY(BlueprintReadOnly, Category="TJ SPACE|Query")
    TArray<FTJSpatialEntityState> Entities;

    UPROPERTY(BlueprintReadOnly, Category="TJ SPACE|Query")
    int32 MatchCount = 0;
};

DECLARE_DYNAMIC_MULTICAST_DELEGATE_OneParam(FTJSpatialEntityChanged, const FTJSpatialEntityState&, Entity);
DECLARE_DYNAMIC_MULTICAST_DELEGATE_OneParam(FTJSpatialDistrictChanged, const FTJSpatialDistrictState&, District);
DECLARE_DYNAMIC_MULTICAST_DELEGATE_OneParam(FTJEnvironmentStateChanged, const FTJSystemHealthState&, Health);

UCLASS(BlueprintType)
class TJSPACE_API UTJSpatialWorldKernel : public UObject
{
    GENERATED_BODY()

public:
    UFUNCTION(BlueprintCallable, Category="TJ SPACE|Citadel")
    bool BootCitadel(const FTJSpatialServerConfig& ServerConfig);

    UFUNCTION(BlueprintCallable, Category="TJ SPACE|Registry")
    bool RegisterSpatialEntity(const FString& BackendId, AActor* Actor);

    UFUNCTION(BlueprintCallable, Category="TJ SPACE|Streaming")
    bool StreamDistrict(const FTJSpatialDistrictCoord& Coord);

    UFUNCTION(BlueprintCallable, Category="TJ SPACE|Environment")
    bool SetEnvironmentState(const FTJSystemHealthState& SystemHealth);

    UFUNCTION(BlueprintCallable, Category="TJ SPACE|Persistence")
    bool PersistWorldLayout(const FTJSpatialWorldLayout& Layout);

    UFUNCTION(BlueprintCallable, Category="TJ SPACE|Query")
    FTJSpatialQueryResult QuerySpatialState(const FTJSpatialQueryFilter& Filter) const;

    UFUNCTION(BlueprintCallable, Category="TJ SPACE|Citadel")
    bool ResetCitadel(bool bPreserveData);

    UFUNCTION(BlueprintPure, Category="TJ SPACE|Registry")
    AActor* ResolveBackendId(const FString& BackendId) const;

    UFUNCTION(BlueprintPure, Category="TJ SPACE|Registry")
    FString ResolveActorBackendId(AActor* Actor) const;

    UFUNCTION(BlueprintPure, Category="TJ SPACE|Environment")
    FTJSystemHealthState GetEnvironmentState() const { return EnvironmentState; }

    UFUNCTION(BlueprintPure, Category="TJ SPACE|Streaming")
    FVector GetStreamingOrigin() const { return StreamingOrigin; }

    UFUNCTION(BlueprintPure, Category="TJ SPACE|Citadel")
    bool IsBooted() const { return bBooted; }

    UPROPERTY(BlueprintAssignable, Category="TJ SPACE|Registry")
    FTJSpatialEntityChanged OnSpatialEntityChanged;

    UPROPERTY(BlueprintAssignable, Category="TJ SPACE|Streaming")
    FTJSpatialDistrictChanged OnSpatialDistrictChanged;

    UPROPERTY(BlueprintAssignable, Category="TJ SPACE|Environment")
    FTJEnvironmentStateChanged OnEnvironmentStateChanged;

private:
    FString MakeDistrictId(const FTJSpatialDistrictCoord& Coord) const;
    FTJSpatialDistrictCoord WorldToDistrict(const FVector& Location) const;
    bool PointInsideFilter(const FVector& Point, const FTJSpatialQueryFilter& Filter) const;
    bool ApplyHealthToActor(AActor* Actor, const FTJSystemHealthState& Health) const;
    bool LoadPersistedLayout();
    bool SavePersistedLayout() const;
    void ClearRegistry();

    UPROPERTY()
    TObjectPtr<UWorld> World;

    UPROPERTY()
    TMap<FString, TObjectPtr<AActor>> BackendToActor;

    TMap<TWeakObjectPtr<AActor>, FString> ActorToBackend;
    TMap<FString, FTJSpatialEntityState> EntityStates;
    TMap<FTJSpatialDistrictCoord, FTJSpatialDistrictState> Districts;

    FTJSpatialServerConfig ServerConfig;
    FTJSystemHealthState EnvironmentState;
    FVector StreamingOrigin = FVector::ZeroVector;
    bool bBooted = false;
    FString PersistencePath;
};