#pragma once
#include "CoreMinimal.h"
#include "GameFramework/Actor.h"
#include "Components/SceneComponent.h"
#include "Components/StaticMeshComponent.h"
#include "ContainerIsolationVisualization.generated.h"

UENUM(BlueprintType)
enum class ETJContainerBoundaryMode : uint8 { Solid, XRay, Isolation, Traffic };

UENUM(BlueprintType)
enum class ETJContainerHealth : uint8 { Unknown, Healthy, Degraded, Critical, Stopped };

USTRUCT(BlueprintType)
struct TJSPACE_API FTJContainerResourceLimit {
    GENERATED_BODY()
    UPROPERTY(EditAnywhere, BlueprintReadWrite) float CpuLimit01=1.0f;
    UPROPERTY(EditAnywhere, BlueprintReadWrite) float MemoryLimit01=1.0f;
    UPROPERTY(EditAnywhere, BlueprintReadWrite) float PidsLimit01=1.0f;
    UPROPERTY(EditAnywhere, BlueprintReadWrite) float IoLimit01=1.0f;
};

USTRUCT(BlueprintType)
struct TJSPACE_API FTJContainerVolumeMount {
    GENERATED_BODY()
    UPROPERTY(EditAnywhere, BlueprintReadWrite) FString HostPath;
    UPROPERTY(EditAnywhere, BlueprintReadWrite) FString ContainerPath;
    UPROPERTY(EditAnywhere, BlueprintReadWrite) bool bReadOnly=false;
};

USTRUCT(BlueprintType)
struct TJSPACE_API FTJContainerManifest {
    GENERATED_BODY()
    UPROPERTY(EditAnywhere, BlueprintReadWrite) FString ContainerId;
    UPROPERTY(EditAnywhere, BlueprintReadWrite) FString PackageId;
    UPROPERTY(EditAnywhere, BlueprintReadWrite) TArray<FString> SubcontainerIds;
    UPROPERTY(EditAnywhere, BlueprintReadWrite) TArray<FTJContainerVolumeMount> VolumeMounts;
    UPROPERTY(EditAnywhere, BlueprintReadWrite) FTJContainerResourceLimit Limits;
    UPROPERTY(EditAnywhere, BlueprintReadWrite) FTJContainerHealth Health=ETJContainerHealth::Unknown;
};

UCLASS()
class TJSPACE_API ATJContainerPod : public AActor {
    GENERATED_BODY()
public:
    ATJContainerPod();
    virtual void Tick(float DeltaSeconds) override;
    void Configure(const FTJContainerManifest& InManifest);
    bool RenderBoundary(ETJContainerBoundaryMode Mode);
    bool RenderSubcontainers();
    bool RenderVolumes();
    bool RenderLimits();
    bool RenderHealth();
    void SetHealth(ETJContainerHealth NewHealth);
    void Archive();

private:
    void SetScalar(UPrimitiveComponent* C,FName P,float V) const;
    UPROPERTY() TObjectPtr<USceneComponent> Root;
    UPROPERTY() TObjectPtr<UStaticMeshComponent> PodShell;
    UPROPERTY() TArray<TObjectPtr<UStaticMeshComponent>> BoundaryLayers;
    UPROPERTY() TArray<TObjectPtr<UStaticMeshComponent>> Subcontainers;
    UPROPERTY() TArray<TObjectPtr<UStaticMeshComponent>> VolumeMounts;
    UPROPERTY() TArray<TObjectPtr<UStaticMeshComponent>> ResourceGovernors;
    UPROPERTY() TArray<TObjectPtr<UStaticMeshComponent>> VitalSigns;
    FTJContainerManifest Manifest;
    ETJContainerBoundaryMode BoundaryMode=ETJContainerBoundaryMode::Solid;
    ETJContainerHealth Health=ETJContainerHealth::Unknown;
    float VitalPhase=0.0f;
};

UCLASS(BlueprintType)
class TJSPACE_API UTJContainerIsolationVisualization : public UObject {
    GENERATED_BODY()
public:
    UFUNCTION(BlueprintCallable) ATJContainerPod* RenderContainerPod(const FString& ContainerId);
    UFUNCTION(BlueprintCallable) bool ShowContainerBoundary(const FString& ContainerId, ETJContainerBoundaryMode Mode);
    UFUNCTION(BlueprintCallable) bool RenderSubcontainers(const FString& PackageId);
    UFUNCTION(BlueprintCallable) bool ShowVolumeMounts(const FString& ContainerId);
    UFUNCTION(BlueprintCallable) bool VisualizeResourceLimits(const FString& ContainerId);
    UFUNCTION(BlueprintCallable) bool ShowContainerHealth(const FString& ContainerId);
    UFUNCTION(BlueprintPure) ATJContainerPod* ResolveContainer(const FString& ContainerId) const;
    void RegisterManifest(const FTJContainerManifest& Manifest);
private:
    UPROPERTY() TMap<FString,FTJContainerManifest> Manifests;
    UPROPERTY() TMap<FString,TObjectPtr<ATJContainerPod>> Pods;
};