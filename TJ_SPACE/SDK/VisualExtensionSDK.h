#pragma once
#include "CoreMinimal.h"
#include "UObject/Object.h"
#include "VisualExtensionSDK.generated.h"

USTRUCT(BlueprintType) struct TJSPACE_API FTJServiceVisualSpec{GENERATED_BODY()
 UPROPERTY(EditAnywhere,BlueprintReadWrite) FString PackageId; UPROPERTY(EditAnywhere,BlueprintReadWrite) FString BuildingMesh; UPROPERTY(EditAnywhere,BlueprintReadWrite) FString MaterialSet; UPROPERTY(EditAnywhere,BlueprintReadWrite) FString VisualVersion;};
USTRUCT(BlueprintType) struct TJSPACE_API FTJCitadelThemeSpec{GENERATED_BODY()
 UPROPERTY(EditAnywhere,BlueprintReadWrite) FString ThemeId; UPROPERTY(EditAnywhere,BlueprintReadWrite) FString Name; UPROPERTY(EditAnywhere,BlueprintReadWrite) FString BaseMaterialSet;};
USTRUCT(BlueprintType) struct TJSPACE_API FTJVisualPluginSpec{GENERATED_BODY()
 UPROPERTY(EditAnywhere,BlueprintReadWrite) FString PluginId; UPROPERTY(EditAnywhere,BlueprintReadWrite) FString Version; UPROPERTY(EditAnywhere,BlueprintReadWrite) FString VisualEntryPoint;};
USTRUCT(BlueprintType) struct TJSPACE_API FTJVisualModManifest{GENERATED_BODY()
 UPROPERTY(EditAnywhere,BlueprintReadWrite) FString ModId; UPROPERTY(EditAnywhere,BlueprintReadWrite) FString PackageId; UPROPERTY(EditAnywhere,BlueprintReadWrite) FString Signature; UPROPERTY(EditAnywhere,BlueprintReadWrite) TArray<FString> Assets;};
USTRUCT(BlueprintType) struct TJSPACE_API FTJComplianceReport{GENERATED_BODY()
 UPROPERTY(BlueprintReadOnly) bool bCompliant=false; UPROPERTY(BlueprintReadOnly) TArray<FString> Errors; UPROPERTY(BlueprintReadOnly) TArray<FString> Warnings; UPROPERTY(BlueprintReadOnly) FString ModId;};
USTRUCT(BlueprintType) struct TJSPACE_API FTJVisualExtensionManifest{GENERATED_BODY()
 UPROPERTY(EditAnywhere,BlueprintReadWrite) FString ExtensionId; UPROPERTY(EditAnywhere,BlueprintReadWrite) FString Version; UPROPERTY(EditAnywhere,BlueprintReadWrite) FString EntryPoint; UPROPERTY(EditAnywhere,BlueprintReadWrite) FString Signature;};
UCLASS(BlueprintType) class TJSPACE_API UTJVisualExtensionSDK:public UObject{GENERATED_BODY()
public:
 UFUNCTION(BlueprintCallable) FTJServiceVisualSpec CreateServiceVisualSpec(const FString&PackageId);
 UFUNCTION(BlueprintCallable) FTJCitadelThemeSpec CreateCitadelTheme(const FTJServiceVisualSpec&Spec);
 UFUNCTION(BlueprintCallable) FTJVisualPluginSpec CreateVisualPlugin(const FTJServiceVisualSpec&Spec);
 UFUNCTION(BlueprintCallable) FTJVisualModManifest PackageVisualMod(const FString&Dir);
 UFUNCTION(BlueprintCallable) bool HotReloadVisual(const FString&PackageId);
 UFUNCTION(BlueprintCallable) FTJComplianceReport ValidateVisualCompliance(const FTJVisualModManifest&Mod);
 UFUNCTION(BlueprintCallable) bool RegisterVisualExtensionPoint(const FTJVisualExtensionManifest&Manifest);
};