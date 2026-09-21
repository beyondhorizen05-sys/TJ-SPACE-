#pragma once
#include "CoreMinimal.h"
#include "GameFramework/Actor.h"
#include "Components/SceneComponent.h"
#include "Components/StaticMeshComponent.h"
#include "TelemetryHealthMetrics.generated.h"

UENUM(BlueprintType) enum class ETJAlertSeverity:uint8{ Info, Warning, Critical };
USTRUCT(BlueprintType) struct TJSPACE_API FTJSystemMetrics{GENERATED_BODY()
 UPROPERTY(EditAnywhere,BlueprintReadWrite) float CPULoad01=0; UPROPERTY(EditAnywhere,BlueprintReadWrite) float MemoryUsage01=0; UPROPERTY(EditAnywhere,BlueprintReadWrite) float DiskUsage01=0; UPROPERTY(EditAnywhere,BlueprintReadWrite) float NetworkThroughput01=0; UPROPERTY(EditAnywhere,BlueprintReadWrite) double UptimeSeconds=0;};
USTRUCT(BlueprintType) struct TJSPACE_API FTJContainerMetrics{GENERATED_BODY()
 UPROPERTY(EditAnywhere,BlueprintReadWrite) FString ContainerId; UPROPERTY(EditAnywhere,BlueprintReadWrite) FTJSystemMetrics Metrics;};
USTRUCT(BlueprintType) struct TJSPACE_API FTJSystemAlert{GENERATED_BODY()
 UPROPERTY(EditAnywhere,BlueprintReadWrite) ETJAlertSeverity Severity=ETJAlertSeverity::Info; UPROPERTY(EditAnywhere,BlueprintReadWrite) FString Source; UPROPERTY(EditAnywhere,BlueprintReadWrite) double TimestampSeconds=0;};
UCLASS() class TJSPACE_API ATJTelemetryCitadel:public AActor{GENERATED_BODY()
public: ATJTelemetryCitadel(); virtual void Tick(float DeltaSeconds) override;
bool RenderSystemMetrics(); bool RenderCPU(); bool RenderMemory(); bool RenderDisk(); bool RenderNetwork(); bool RenderUptime(); bool TriggerAlert(ETJAlertSeverity Severity,const FString&Source); bool RenderContainer(const FString&ContainerId);
void SetSystemMetrics(const FTJSystemMetrics&Metrics); void SetContainerMetrics(const FTJContainerMetrics&Metrics);
private:
UPROPERTY() TObjectPtr<USceneComponent> Root; UPROPERTY() TObjectPtr<UStaticMeshComponent> PowerPlant; UPROPERTY() TObjectPtr<UStaticMeshComponent> PressureVessels; UPROPERTY() TObjectPtr<UStaticMeshComponent> StorageSilos; UPROPERTY() TObjectPtr<UStaticMeshComponent> NetworkExchange; UPROPERTY() TObjectPtr<UStaticMeshComponent> UptimeMonument; UPROPERTY() TArray<TObjectPtr<UStaticMeshComponent>> AlertEvents; UPROPERTY() TArray<TObjectPtr<UStaticMeshComponent>> ContainerFacilities;
FTJSystemMetrics SystemMetrics; TMap<FString,FTJContainerMetrics> Containers; TArray<FTJSystemAlert> Alerts; double RuntimeSeconds=0;
void SetScalar(UPrimitiveComponent*C,FName P,float V) const; void ApplyFacilityState(); float Strain(float V) const;};
UCLASS(BlueprintType) class TJSPACE_API UTJTelemetryHealthMetrics:public UObject{GENERATED_BODY()
public:
UFUNCTION(BlueprintCallable) ATJTelemetryCitadel* BuildTelemetryCitadel();
UFUNCTION(BlueprintCallable) bool RenderSystemMetrics(); UFUNCTION(BlueprintCallable) bool RenderCPULoad(); UFUNCTION(BlueprintCallable) bool RenderMemoryUsage(); UFUNCTION(BlueprintCallable) bool RenderDiskUsage(); UFUNCTION(BlueprintCallable) bool RenderNetworkThroughput(); UFUNCTION(BlueprintCallable) bool RenderUptime(); UFUNCTION(BlueprintCallable) bool TriggerSystemAlert(ETJAlertSeverity Severity,const FString&Source); UFUNCTION(BlueprintCallable) bool RenderContainerMetrics(const FString&ContainerId);
void SetSystemMetrics(const FTJSystemMetrics&Metrics); void SetContainerMetrics(const FTJContainerMetrics&Metrics);
private: UPROPERTY() TObjectPtr<ATJTelemetryCitadel> Citadel;};