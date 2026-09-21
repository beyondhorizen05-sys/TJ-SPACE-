#pragma once
#include "CoreMinimal.h"
#include "Internationalization/Internationalization.h"
#include "AccessibilityLocalizationSystem.generated.h"

UENUM(BlueprintType) enum class ETJColorblindMode:uint8 { None, Protanopia, Deuteranopia, Tritanopia, Achromatopsia };
UENUM(BlueprintType) enum class ETJCaptionChannel:uint8 { Dialogue, Effects, Ambient, Alerts, Voice };
USTRUCT(BlueprintType) struct TJSPACE_API FTJKeyboardLayout { GENERATED_BODY()
 UPROPERTY(EditAnywhere,BlueprintReadWrite) FString LayoutId; UPROPERTY(EditAnywhere,BlueprintReadWrite) FString DisplayName; UPROPERTY(EditAnywhere,BlueprintReadWrite) FString CultureCode;
};
USTRUCT(BlueprintType) struct TJSPACE_API FTJMotionComfortOptions { GENERATED_BODY()
 UPROPERTY(EditAnywhere,BlueprintReadWrite) bool bSnapTurn=true; UPROPERTY(EditAnywhere,BlueprintReadWrite) bool bTeleportLocomotion=true; UPROPERTY(EditAnywhere,BlueprintReadWrite) bool bVignette=true; UPROPERTY(EditAnywhere,BlueprintReadWrite) bool bFOVClamp=true; UPROPERTY(EditAnywhere,BlueprintReadWrite) float MaxFOV=90.f;
};
USTRUCT(BlueprintType) struct TJSPACE_API FTJCaptionStyle { GENERATED_BODY()
 UPROPERTY(EditAnywhere,BlueprintReadWrite) float TextScale=1.f; UPROPERTY(EditAnywhere,BlueprintReadWrite) float BackgroundOpacity=.65f; UPROPERTY(EditAnywhere,BlueprintReadWrite) bool bSpeakerLabels=true; UPROPERTY(EditAnywhere,BlueprintReadWrite) bool bSoundDescriptions=true;
};
USTRUCT(BlueprintType) struct TJSPACE_API FTJStatusColorMap { GENERATED_BODY()
 UPROPERTY(BlueprintReadOnly) FLinearColor Info; UPROPERTY(BlueprintReadOnly) FLinearColor Warning; UPROPERTY(BlueprintReadOnly) FLinearColor Critical; UPROPERTY(BlueprintReadOnly) FLinearColor Healthy; UPROPERTY(BlueprintReadOnly) FLinearColor Denied; UPROPERTY(BlueprintReadOnly) FLinearColor Authorized;
};
UCLASS(BlueprintType) class TJSPACE_API UTJAccessibilityLocalizationSystem:public UObject { GENERATED_BODY()
public:
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Accessibility") bool LoadLocale(const FString& LocaleCode);
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Accessibility") bool ApplyLocale(const FString& LocaleCode);
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Accessibility") TArray<FString> GetAvailableLocales() const;
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Accessibility") bool RegisterKeyboardLayout(const FTJKeyboardLayout& Layout);
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Accessibility") bool SetActiveLayout(const FString& Layout);
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Accessibility") bool EnableColorblindMode(ETJColorblindMode Mode);
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Accessibility") bool ConfigureMotionComfort(const FTJMotionComfortOptions& Options);
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Accessibility") FText DescribeEntity(AActor* Actor) const;
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Accessibility") bool EnableSubtitles(ETJCaptionChannel Channel);
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Accessibility") bool ConfigureCaptionStyle(const FTJCaptionStyle& Style);
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Accessibility") bool ScaleUIText(float Scale);
 UFUNCTION(BlueprintPure,Category="TJ SPACE|Accessibility") FTJStatusColorMap GetStatusColorMap() const;
 UFUNCTION(BlueprintPure,Category="TJ SPACE|Accessibility") FText LocalizeKey(const FString& Namespace,const FString& Key,const FString& Fallback) const;
 UFUNCTION(BlueprintPure,Category="TJ SPACE|Accessibility") bool IsSubtitleChannelEnabled(ETJCaptionChannel Channel) const;
 UFUNCTION(BlueprintPure,Category="TJ SPACE|Accessibility") FTJMotionComfortOptions GetMotionComfort() const{return Motion;}
private:
 FString ResolveLocaleFallback(const FString& Requested) const;
 void BuildStatusMap();
 UPROPERTY() TMap<FString,FTJKeyboardLayout> Layouts;
 UPROPERTY() TSet<ETJCaptionChannel> SubtitleChannels;
 FString ActiveLocale="en"; FString ActiveLayoutId; ETJColorblindMode ColorMode=ETJColorblindMode::None;
 FTJMotionComfortOptions Motion; FTJCaptionStyle CaptionStyle; FTJStatusColorMap Colors; float UITextScale=1.f;
};