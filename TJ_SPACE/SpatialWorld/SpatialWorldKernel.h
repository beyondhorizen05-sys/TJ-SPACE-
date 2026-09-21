#pragma once

#include "CoreMinimal.h"
#include "UObject/Object.h"
#include "SpatialWorldKernel.generated.h"

USTRUCT(BlueprintType)
struct TJSPACE_API FTJSpatialCoordinate
{
    GENERATED_BODY()

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Spatial")
    FVector Location = FVector::ZeroVector;
};

USTRUCT(BlueprintType)
struct TJSPACE_API FTJSpatialBounds
{
    GENERATED_BODY()

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Spatial")
    FVector Center = FVector::ZeroVector;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Spatial")
    FVector Extent = FVector(100.0, 100.0, 100.0);

    bool Contains(const FVector& Point) const
    {
        const FVector Delta = Point - Center;
        return FMath::Abs(Delta.X) <= Extent.X &&
               FMath::Abs(Delta.Y) <= Extent.Y &&
               FMath::Abs(Delta.Z) <= Extent.Z;
    }
};

USTRUCT(BlueprintType)
struct TJSPACE_API FTJSpatialZone
{
    GENERATED_BODY()

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Spatial")
    FString Id;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Spatial")
    FTJSpatialBounds Bounds;

    UPROPERTY(VisibleAnywhere, BlueprintReadOnly, Category="TJ SPACE|Spatial")
    bool bLoaded = false;
};

DECLARE_DYNAMIC_MULTICAST_DELEGATE_OneParam(FTJSpatialZoneStateChanged, const FTJSpatialZone&, Zone);

UCLASS(BlueprintType)
class TJSPACE_API UTJSpatialWorldKernel : public UObject
{
    GENERATED_BODY()

public:
    UFUNCTION(BlueprintCallable, Category="TJ SPACE|Spatial")
    bool InitializeWorldKernel(UWorld* WorldContext);

    UFUNCTION(BlueprintCallable, Category="TJ SPACE|Spatial")
    bool RegisterSpatialZone(const FTJSpatialZone& Zone);

    UFUNCTION(BlueprintPure, Category="TJ SPACE|Spatial")
    bool ResolveZone(const FVector& WorldLocation, FTJSpatialZone& OutZone) const;

    UFUNCTION(BlueprintCallable, Category="TJ SPACE|Spatial")
    bool SetStreamingOrigin(const FVector& Origin);

    UFUNCTION(BlueprintCallable, Category="TJ SPACE|Spatial")
    int32 UpdateStreamingState();

    UFUNCTION(BlueprintPure, Category="TJ SPACE|Spatial")
    FVector GetStreamingOrigin() const { return StreamingOrigin; }

    UFUNCTION(BlueprintPure, Category="TJ SPACE|Spatial")
    int32 GetZoneCount() const { return Zones.Num(); }

    UPROPERTY(BlueprintAssignable, Category="TJ SPACE|Spatial")
    FTJSpatialZoneStateChanged OnZoneStateChanged;

private:
    UPROPERTY()
    TObjectPtr<UWorld> World;

    UPROPERTY()
    TArray<FTJSpatialZone> Zones;

    FVector StreamingOrigin = FVector::ZeroVector;
    double LoadingRange = 76800.0;
};