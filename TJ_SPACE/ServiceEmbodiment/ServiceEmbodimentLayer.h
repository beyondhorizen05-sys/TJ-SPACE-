#pragma once

#include "CoreMinimal.h"
#include "GameFramework/Actor.h"
#include "Components/SceneComponent.h"
#include "Components/StaticMeshComponent.h"
#include "Components/BoxComponent.h"
#include "Engine/StreamableManager.h"
#include "ServiceEmbodimentLayer.generated.h"

UENUM(BlueprintType)
enum class ETJServiceState : uint8
{
    Stopped,
    Starting,
    Running,
    Error,
    Updating
};

UENUM(BlueprintType)
enum class ETJServiceTransitionCurve : uint8
{
    Linear,
    CubicEaseInOut,
    QuinticEaseOut,
    CubicEaseOut
};

USTRUCT(BlueprintType)
struct TJSPACE_API FTJServiceInterfaceManifest
{
    GENERATED_BODY()

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Service")
    FString InterfaceId;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Service")
    FString Protocol;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Service")
    int32 Port = 0;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Service")
    FString DisplayName;
};

USTRUCT(BlueprintType)
struct TJSPACE_API FTJServiceDependencyManifest
{
    GENERATED_BODY()

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Service")
    FString DependencyId;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Service")
    FString DependencyType;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Service")
    double Throughput01 = 0.0;
};

USTRUCT(BlueprintType)
struct TJSPACE_API FTJServiceResourceManifest
{
    GENERATED_BODY()

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Service")
    double CpuCores = 0.0;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Service")
    double MemoryGB = 0.0;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Service")
    double StorageGB = 0.0;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Service")
    double NetworkMbps = 0.0;
};

USTRUCT(BlueprintType)
struct TJSPACE_API FTJServiceManifest
{
    GENERATED_BODY()

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Service")
    FString PackageId;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Service")
    FString DisplayName;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Service")
    FSoftObjectPath BuildingMesh;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Service")
    FSoftObjectPath InteriorMesh;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Service")
    TArray<FSoftObjectPath> InteriorModuleMeshes;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Service")
    TArray<FTJServiceInterfaceManifest> Interfaces;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Service")
    TArray<FTJServiceDependencyManifest> Dependencies;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Service")
    FTJServiceResourceManifest Resources;

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Service")
    FVector BuildingScale = FVector(1.0);

    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category="TJ SPACE|Service")
    FVector InteriorScale = FVector(1.0);
};

USTRUCT(BlueprintType)
struct TJSPACE_API FTJServiceTransitionSpec
{
    GENERATED_BODY()

    UPROPERTY(BlueprintReadOnly, Category="TJ SPACE|Service")
    ETJServiceState From = ETJServiceState::Stopped;

    UPROPERTY(BlueprintReadOnly, Category="TJ SPACE|Service")
    ETJServiceState To = ETJServiceState::Stopped;

    UPROPERTY(BlueprintReadOnly, Category="TJ SPACE|Service")
    ETJServiceTransitionCurve Curve = ETJServiceTransitionCurve::CubicEaseInOut;

    UPROPERTY(BlueprintReadOnly, Category="TJ SPACE|Service")
    float DurationSeconds = 0.0f;
};

UCLASS()
class TJSPACE_API ATJServiceBuilding : public AActor
{
    GENERATED_BODY()

public:
    ATJServiceBuilding();

    virtual void Tick(float DeltaSeconds) override;

    void ConfigureManifest(const FTJServiceManifest& InManifest);
    bool BuildShell();
    bool BuildInterior();
    bool BuildInterfaces();
    bool BuildDependencies();
    bool BuildResourceFootprint();

    void BeginStateTransition(ETJServiceState NewState);
    void Archive();

    ETJServiceState GetServiceState() const { return CurrentState; }
    const FTJServiceManifest& GetManifest() const { return Manifest; }
    const FTJServiceTransitionSpec& GetActiveTransition() const { return ActiveTransition; }
    bool IsArchived() const { return bArchived; }

private:
    static float EvaluateCurve(ETJServiceTransitionCurve Curve, float Alpha);
    static FTJServiceTransitionSpec MakeTransition(ETJServiceState From, ETJServiceState To);
    void ApplyArchitecturalState(float Alpha);
    void SetMaterialScalar(UPrimitiveComponent* Component, FName Parameter, float Value) const;

    UPROPERTY()
    TObjectPtr<USceneComponent> Root;

    UPROPERTY()
    TObjectPtr<UStaticMeshComponent> Exterior;

    UPROPERTY()
    TObjectPtr<USceneComponent> InteriorRoot;

    UPROPERTY()
    TArray<TObjectPtr<UStaticMeshComponent>> InteriorModules;

    UPROPERTY()
    TArray<TObjectPtr<UStaticMeshComponent>> InterfaceExits;

    UPROPERTY()
    TArray<TObjectPtr<UStaticMeshComponent>> DependencyPipes;

    UPROPERTY()
    TArray<TObjectPtr<UStaticMeshComponent>> ResourceFootprint;

    FTJServiceManifest Manifest;
    ETJServiceState CurrentState = ETJServiceState::Stopped;
    ETJServiceState TargetState = ETJServiceState::Stopped;
    FTJServiceTransitionSpec ActiveTransition;
    float TransitionElapsed = 0.0f;
    bool bTransitioning = false;
    bool bArchived = false;
    FVector BaseScale = FVector::OneVector;
};

DECLARE_DYNAMIC_MULTICAST_DELEGATE_TwoParams(FTJServiceStateTransitioned, ETJServiceState, From, ETJServiceState, To);
DECLARE_DYNAMIC_MULTICAST_DELEGATE_TwoParams(FTJServiceBuildingArchived, const FString&, PackageId, bool, bPreserved);

UCLASS(BlueprintType)
class TJSPACE_API UTJServiceEmbodimentLayer : public UObject
{
    GENERATED_BODY()

public:
    UFUNCTION(BlueprintCallable, Category="TJ SPACE|Service")
    ATJServiceBuilding* InstantiateServiceBuilding(const FString& PackageId, const FTJServiceManifest& Manifest);

    UFUNCTION(BlueprintCallable, Category="TJ SPACE|Service")
    bool BindServiceState(const FString& PackageId, ETJServiceState State);

    UFUNCTION(BlueprintCallable, Category="TJ SPACE|Service")
    bool RenderServiceInterior(const FString& PackageId);

    UFUNCTION(BlueprintCallable, Category="TJ SPACE|Service")
    bool DisplayServiceInterfaces(const FString& PackageId);

    UFUNCTION(BlueprintCallable, Category="TJ SPACE|Service")
    bool ShowServiceDependencies(const FString& PackageId);

    UFUNCTION(BlueprintCallable, Category="TJ SPACE|Service")
    bool ShowResourceDraw(const FString& PackageId);

    UFUNCTION(BlueprintCallable, Category="TJ SPACE|Service")
    bool ArchiveServiceBuilding(const FString& PackageId);

    UFUNCTION(BlueprintPure, Category="TJ SPACE|Service")
    ATJServiceBuilding* ResolveServiceBuilding(const FString& PackageId) const;

    UPROPERTY(BlueprintAssignable, Category="TJ SPACE|Service")
    FTJServiceStateTransitioned OnServiceStateTransitioned;

    UPROPERTY(BlueprintAssignable, Category="TJ SPACE|Service")
    FTJServiceBuildingArchived OnServiceBuildingArchived;

private:
    UPROPERTY()
    TMap<FString, TObjectPtr<ATJServiceBuilding>> Buildings;
};