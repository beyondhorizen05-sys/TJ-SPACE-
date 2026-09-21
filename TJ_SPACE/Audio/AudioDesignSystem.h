#pragma once
#include "CoreMinimal.h"
#include "Sound/SoundBase.h"
#include "Sound/ReverbEffect.h"
#include "AudioDesignSystem.generated.h"

UENUM(BlueprintType) enum class ETJAudioChannel : uint8 { Master, Ambience, Machinery, UI, Voice, Alerts, Footsteps };
UENUM(BlueprintType) enum class ETJAlarmSeverity : uint8 { Info, Warning, Critical };

USTRUCT(BlueprintType) struct TJSPACE_API FTJSpatialAudioParams { GENERATED_BODY()
 UPROPERTY(EditAnywhere,BlueprintReadWrite) float Volume=1.f;
 UPROPERTY(EditAnywhere,BlueprintReadWrite) float Pitch=1.f;
 UPROPERTY(EditAnywhere,BlueprintReadWrite) float StartTime=0.f;
 UPROPERTY(EditAnywhere,BlueprintReadWrite) bool bLooping=false;
 UPROPERTY(EditAnywhere,BlueprintReadWrite) TObjectPtr<USoundAttenuation> Attenuation=nullptr;
};
USTRUCT(BlueprintType) struct TJSPACE_API FTJAudioReverbProfile { GENERATED_BODY()
 UPROPERTY(EditAnywhere,BlueprintReadWrite) FName ProfileId;
 UPROPERTY(EditAnywhere,BlueprintReadWrite) TObjectPtr<UReverbEffect> ReverbEffect=nullptr;
 UPROPERTY(EditAnywhere,BlueprintReadWrite) float Volume=1.f;
 UPROPERTY(EditAnywhere,BlueprintReadWrite) float FadeTime=.75f;
 UPROPERTY(EditAnywhere,BlueprintReadWrite) int32 Priority=100;
};
USTRUCT(BlueprintType) struct TJSPACE_API FTJAudioZoneState { GENERATED_BODY()
 UPROPERTY(BlueprintReadOnly) FString ZoneId;
 UPROPERTY(BlueprintReadOnly) TObjectPtr<USoundBase> Bed=nullptr;
 UPROPERTY(BlueprintReadOnly) TObjectPtr<UAudioComponent> Component=nullptr;
};
USTRUCT(BlueprintType) struct TJSPACE_API FTJVoiceStreamBinding { GENERATED_BODY()
 UPROPERTY(BlueprintReadOnly) FString AgentId;
 UPROPERTY(BlueprintReadOnly) int64 ByteCount=0;
 UPROPERTY(BlueprintReadOnly) bool bBound=false;
};

UCLASS(BlueprintType) class TJSPACE_API UTJAudioDesignSystem : public UObject { GENERATED_BODY()
public:
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Audio") bool LoadAmbienceBed(const FString& ZoneId,USoundBase* BedAsset);
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Audio") bool SetAudioZone(const FString& ZoneId);
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Audio") UAudioComponent* PlaySpatialSound(AActor* Actor,USoundBase* Cue,const FTJSpatialAudioParams& Params);
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Audio") bool PlayUISound(UObject* Panel,USoundBase* Cue);
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Audio") bool DuckForDialog(const FString& ConversationId,float Amount);
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Audio") bool SetMasterVolume(ETJAudioChannel Channel,float Value);
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Audio") bool ApplyAcousticReverb(const FString& ZoneId,const FTJAudioReverbProfile& ReverbProfile);
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Audio") bool StreamVoice(const FString& AgentId,const TArray<uint8>& AudioStream);
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Audio") bool TriggerAlarm(ETJAlarmSeverity Severity,const FString& Source);
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Audio|Integration") bool BindServiceStateAudio(const FString& PackageId);
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Audio|Integration") bool ValidateServiceTransitionCueCoverage() const;
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Audio|Integration") bool RegisterServiceTransitionCue(const FString& TransitionKey,USoundBase* Cue);
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Audio|Integration") bool RegisterAlertSignature(ETJAlarmSeverity Severity,const FString& SourceKey,USoundBase* Cue);
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Audio|Integration") bool BindTelemetryAlertSignature(ETJAlarmSeverity Severity,const FString& SourceKey);
 void SetWorld(UWorld* InWorld);
private:
 UFUNCTION() void OnServiceTransition(ETJServiceState From,ETJServiceState To);
 FString MakeTransitionKey(ETJServiceState From,ETJServiceState To) const;
 FString MakeAlertKey(ETJAlarmSeverity Severity,const FString& Source) const;
 USoundBase* ResolveAlertSignature(ETJAlarmSeverity Severity,const FString& Source) const;
 bool ApplyChannelVolume(ETJAudioChannel Channel,float Value);
 UPROPERTY() TObjectPtr<UWorld> World;
 UPROPERTY() TMap<FString,TObjectPtr<USoundBase>> AmbienceBeds;
 UPROPERTY() TMap<FString,TObjectPtr<UAudioComponent>> ZoneComponents;
 UPROPERTY() TMap<FString,FTJAudioReverbProfile> ReverbProfiles;
 UPROPERTY() TMap<FString,TObjectPtr<USoundBase>> TransitionCues;
 UPROPERTY() TMap<FString,TObjectPtr<USoundBase>> AlertCues;
 UPROPERTY() TObjectPtr<USoundMix> DuckMix;
 UPROPERTY() TMap<ETJAudioChannel,TObjectPtr<USoundClass>> ChannelClasses;
 UPROPERTY() TMap<FString,FTJVoiceStreamBinding> VoiceBindings;
 UPROPERTY() TMap<FString,float> ConversationDucks;
 UPROPERTY() FString ActiveZone;
 float CrossfadeSeconds=1.5f;
};