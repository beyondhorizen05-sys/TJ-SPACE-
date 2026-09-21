#pragma once
#include "CoreMinimal.h"
#include "TJSpaceVisualFoundation.generated.h"

USTRUCT(BlueprintType)
struct FTJSpaceArtBible { GENERATED_BODY() UPROPERTY(EditAnywhere,BlueprintReadWrite) FString Version=TEXT("1.0.0"); UPROPERTY(EditAnywhere,BlueprintReadWrite) FString Renderer=TEXT("Unreal Engine 5"); UPROPERTY(EditAnywhere,BlueprintReadWrite) FString Geometry=TEXT("Nanite"); UPROPERTY(EditAnywhere,BlueprintReadWrite) FString Lighting=TEXT("Lumen"); UPROPERTY(EditAnywhere,BlueprintReadWrite) FString ToneMapping=TEXT("ACES"); UPROPERTY(EditAnywhere,BlueprintReadWrite) FString Shading=TEXT("PBR"); };

UCLASS(BlueprintType)
class UTJSpaceVisualFoundation : public UObject { GENERATED_BODY() public:
UFUNCTION(BlueprintCallable,Category="TJ SPACE|System 1") static bool DefineArtDirection(const FTJSpaceArtBible& Bible);
UFUNCTION(BlueprintCallable,Category="TJ SPACE|System 1") static bool BuildMaterialLibrary();
UFUNCTION(BlueprintCallable,Category="TJ SPACE|System 1") static bool ConfigureRenderingPipeline();
UFUNCTION(BlueprintCallable,Category="TJ SPACE|System 1") static bool BuildUIDesignSystem();
};