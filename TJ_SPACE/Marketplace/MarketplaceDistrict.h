#pragma once
#include "CoreMinimal.h"
#include "GameFramework/Actor.h"
#include "Components/SceneComponent.h"
#include "Components/StaticMeshComponent.h"
#include "MarketplaceDistrict.generated.h"

UENUM(BlueprintType) enum class ETJPackageVerification : uint8 { Unknown, Verified, Unverified, Invalid };
UENUM(BlueprintType) enum class ETJInstallStage : uint8 { Inspection, Quarantined, Staged, Installing, Installed, Failed, Removing, CustodyHold };
UENUM(BlueprintType) enum class ETJRemovalStage : uint8 { Armed, Holding, DataCustody, Destroyed };

USTRUCT(BlueprintType) struct TJSPACE_API FTJPackageManifest { GENERATED_BODY()
 UPROPERTY(EditAnywhere,BlueprintReadWrite) FString PackageId;
 UPROPERTY(EditAnywhere,BlueprintReadWrite) FString DisplayName;
 UPROPERTY(EditAnywhere,BlueprintReadWrite) FString Version;
 UPROPERTY(EditAnywhere,BlueprintReadWrite) FString Description;
 UPROPERTY(EditAnywhere,BlueprintReadWrite) FString RegistryUrl;
 UPROPERTY(EditAnywhere,BlueprintReadWrite) FString SignatureBase64;
 UPROPERTY(EditAnywhere,BlueprintReadWrite) FString PublicKeyBase64;
 UPROPERTY(EditAnywhere,BlueprintReadWrite) ETJPackageVerification Verification=ETJPackageVerification::Unknown;
 UPROPERTY(EditAnywhere,BlueprintReadWrite) bool bDataCustodyRequired=true;
};
USTRUCT(BlueprintType) struct TJSPACE_API FTJRegistrySource { GENERATED_BODY()
 UPROPERTY(EditAnywhere,BlueprintReadWrite) FString RegistryId;
 UPROPERTY(EditAnywhere,BlueprintReadWrite) FString RegistryUrl;
 UPROPERTY(EditAnywhere,BlueprintReadWrite) bool bEnabled=true;
 UPROPERTY(EditAnywhere,BlueprintReadWrite) bool bTrusted=false;
};
USTRUCT(BlueprintType) struct TJSPACE_API FTJTargetPlot { GENERATED_BODY()
 UPROPERTY(EditAnywhere,BlueprintReadWrite) FString PlotId;
 UPROPERTY(EditAnywhere,BlueprintReadWrite) FVector Location=FVector::ZeroVector;
};
USTRUCT(BlueprintType) struct TJSPACE_API FTJRemovalState { GENERATED_BODY()
 UPROPERTY(EditAnywhere,BlueprintReadWrite) FString PackageId;
 UPROPERTY(EditAnywhere,BlueprintReadWrite) ETJRemovalStage Stage=ETJRemovalStage::Armed;
 UPROPERTY(EditAnywhere,BlueprintReadWrite) float LeverHoldSeconds=0.0f;
 UPROPERTY(EditAnywhere,BlueprintReadWrite) bool bDataDestroyed=false;
};

UCLASS() class TJSPACE_API ATJMarketplaceDistrict : public AActor {
 GENERATED_BODY()
public:
 ATJMarketplaceDistrict();
 virtual void Tick(float DeltaSeconds) override;
 bool BuildHall();
 bool RenderCatalog(const FString& RegistryUrl);
 bool Inspect(const FString& PackageId);
 bool Install(const FString& PackageId,const FTJTargetPlot& Plot);
 bool Verify(const FString& PackageId);
 bool Update(const FString& PackageId,const FString& Version);
 bool Remove(const FString& PackageId);
 bool HoldRemovalLever(const FString& PackageId,float DeltaSeconds);
 bool ReleaseRemovalLever(const FString& PackageId);
 bool AddRegistry(const FTJRegistrySource& Source);
 bool RemoveRegistry(const FString& RegistryId);
 void RegisterPackage(const FTJPackageManifest& Manifest);
private:
 UPROPERTY() TObjectPtr<USceneComponent> Root;
 UPROPERTY() TObjectPtr<UStaticMeshComponent> Hall;
 UPROPERTY() TArray<TObjectPtr<UStaticMeshComponent>> CatalogPanels;
 UPROPERTY() TArray<TObjectPtr<UStaticMeshComponent>> InspectionStands;
 UPROPERTY() TArray<TObjectPtr<UStaticMeshComponent>> InstallationStages;
 UPROPERTY() TArray<TObjectPtr<UStaticMeshComponent>> SignatureSeals;
 UPROPERTY() TArray<TObjectPtr<UStaticMeshComponent>> QuarantineBarriers;
 UPROPERTY() TArray<TObjectPtr<UStaticMeshComponent>> RenovationStages;
 UPROPERTY() TArray<TObjectPtr<UStaticMeshComponent>> CustodyLevers;
 TMap<FString,FTJPackageManifest> Packages;
 TMap<FString,FTJRegistrySource> Registries;
 TMap<FString,FTJRemovalState> Removals;
 void SetScalar(UPrimitiveComponent* C,FName P,float V) const;
 void RenderQuarantine(const FString& PackageId,bool bQuarantine);
};

UCLASS(BlueprintType) class TJSPACE_API UTJMarketplaceDistrict : public UObject {
 GENERATED_BODY()
public:
 UFUNCTION(BlueprintCallable) ATJMarketplaceDistrict* BuildMarketplaceDistrict();
 UFUNCTION(BlueprintCallable) bool RenderRegistryCatalog(const FString& RegistryUrl);
 UFUNCTION(BlueprintCallable) bool InspectPackage(const FString& PackageId);
 UFUNCTION(BlueprintCallable) bool InstallPackage(const FString& PackageId,const FTJTargetPlot& TargetPlot);
 UFUNCTION(BlueprintCallable) bool VerifyPackageSignature(const FString& PackageId);
 UFUNCTION(BlueprintCallable) bool UpdatePackage(const FString& PackageId,const FString& Version);
 UFUNCTION(BlueprintCallable) bool RemovePackage(const FString& PackageId);
 UFUNCTION(BlueprintCallable) bool ManageRegistrySources();
 UFUNCTION(BlueprintCallable) bool HoldRemovalLever(const FString& PackageId,float DeltaSeconds);
 UFUNCTION(BlueprintCallable) bool ReleaseRemovalLever(const FString& PackageId);
 void RegisterPackage(const FTJPackageManifest& Manifest);
 void RegisterRegistry(const FTJRegistrySource& Source);
private:
 UPROPERTY() TObjectPtr<ATJMarketplaceDistrict> District;
};