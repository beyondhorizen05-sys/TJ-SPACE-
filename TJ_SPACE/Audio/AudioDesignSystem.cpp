#include "AudioDesignSystem.h"
#include "Engine/World.h"
#include "EngineUtils.h"
#include "Kismet/GameplayStatics.h"
#include "Components/AudioComponent.h"
#include "Sound/SoundMix.h"
#include "Sound/SoundClass.h"
#include "AudioDevice.h"
#include "ServiceEmbodimentLayer.h"

void UTJAudioDesignSystem::SetWorld(UWorld* InWorld){World=InWorld;}
bool UTJAudioDesignSystem::LoadAmbienceBed(const FString& ZoneId,USoundBase* BedAsset){if(ZoneId.IsEmpty()||!IsValid(BedAsset))return false;AmbienceBeds.Add(ZoneId,BedAsset);return true;}
bool UTJAudioDesignSystem::SetAudioZone(const FString& ZoneId){
 if(!World||!AmbienceBeds.Contains(ZoneId))return false;
 if(UAudioComponent* Old=ZoneComponents.FindRef(ActiveZone))if(Old)Old->FadeOut(CrossfadeSeconds,0.f,EAudioFaderCurve::SCurve);
 UAudioComponent* NewComp=ZoneComponents.FindRef(ZoneId);
 if(!NewComp){NewComp=UGameplayStatics::SpawnSound2D(World,AmbienceBeds[ZoneId],0.f,1.f,0.f,nullptr,false,false);if(!NewComp)return false;ZoneComponents.Add(ZoneId,NewComp);}
 NewComp->SetSound(AmbienceBeds[ZoneId]);NewComp->FadeIn(CrossfadeSeconds,1.f,0.f,EAudioFaderCurve::SCurve);ActiveZone=ZoneId;return true;
}
UAudioComponent* UTJAudioDesignSystem::PlaySpatialSound(AActor* Actor,USoundBase* Cue,const FTJSpatialAudioParams& P){
 if(!World||!IsValid(Actor)||!IsValid(Cue)||!Actor->GetRootComponent())return nullptr;
 return UGameplayStatics::SpawnSoundAttached(Cue,Actor->GetRootComponent(),NAME_None,FVector::ZeroVector,EAttachLocation::KeepRelativeOffset,true,P.Volume,P.Pitch,P.StartTime,P.Attenuation,nullptr,!P.bLooping);
}
bool UTJAudioDesignSystem::PlayUISound(UObject* Panel,USoundBase* Cue){if(!World||!IsValid(Panel)||!IsValid(Cue))return false;UGameplayStatics::PlaySound2D(World,Cue,1.f,1.f,0.f,nullptr,nullptr,true);return true;}
bool UTJAudioDesignSystem::DuckForDialog(const FString& Id,float Amount){
 if(!World||Id.IsEmpty())return false;const float D=FMath::Clamp(Amount,0.f,1.f);ConversationDucks.Add(Id,D);
 if(!DuckMix)DuckMix=NewObject<USoundMix>(this,TEXT("TJDialogDuckMix"));
 for(const TPair<ETJAudioChannel,TObjectPtr<USoundClass>>& Pair:ChannelClasses){if(Pair.Key==ETJAudioChannel::Voice||Pair.Key==ETJAudioChannel::Master||!IsValid(Pair.Value))continue;UGameplayStatics::SetSoundMixClassOverride(World,DuckMix,Pair.Value,1.f-D,1.f,.15f,true);}
 UGameplayStatics::PushSoundMixModifier(World,DuckMix);return true;
}
bool UTJAudioDesignSystem::ApplyChannelVolume(ETJAudioChannel Channel,float Value){
 if(!World)return false;const float V=FMath::Clamp(Value,0.f,1.f);
 if(Channel==ETJAudioChannel::Master){if(FAudioDevice* Device=World->GetAudioDeviceRaw()){Device->SetTransientPrimaryVolume(V);return true;}return false;}
 USoundClass* Class=ChannelClasses.FindRef(Channel);if(!IsValid(Class))return false;
 if(!DuckMix)DuckMix=NewObject<USoundMix>(this,TEXT("TJChannelMix"));
 UGameplayStatics::SetSoundMixClassOverride(World,DuckMix,Class,V,1.f,.05f,true);UGameplayStatics::PushSoundMixModifier(World,DuckMix);return true;
}
bool UTJAudioDesignSystem::SetMasterVolume(ETJAudioChannel Channel,float Value){return ApplyChannelVolume(Channel,Value);}
bool UTJAudioDesignSystem::ApplyAcousticReverb(const FString& ZoneId,const FTJAudioReverbProfile& Profile){
 if(!World||ZoneId.IsEmpty()||!IsValid(Profile.ReverbEffect))return false;ReverbProfiles.Add(ZoneId,Profile);
 if(FAudioDevice* Device=World->GetAudioDeviceRaw()){Device->ActivateReverbEffect(Profile.ReverbEffect,FName(*ZoneId),Profile.Priority,Profile.Volume,Profile.FadeTime);return true;}return false;
}
bool UTJAudioDesignSystem::StreamVoice(const FString& AgentId,const TArray<uint8>& AudioStream){
 if(AgentId.IsEmpty()||AudioStream.IsEmpty())return false;FTJVoiceStreamBinding B;B.AgentId=AgentId;B.ByteCount=AudioStream.Num();B.bBound=true;VoiceBindings.Add(AgentId,B);return true;
}
FString UTJAudioDesignSystem::MakeAlertKey(ETJAlarmSeverity S,const FString& Source)const{return FString::Printf(TEXT("%d:%s"),(int32)S,*Source);}
FString UTJAudioDesignSystem::MakeTransitionKey(ETJServiceState A,ETJServiceState B)const{return FString::Printf(TEXT("%d>%d"),(int32)A,(int32)B);}
bool UTJAudioDesignSystem::RegisterServiceTransitionCue(const FString& Key,USoundBase* Cue){if(Key.IsEmpty()||!IsValid(Cue))return false;TransitionCues.Add(Key,Cue);return true;}
bool UTJAudioDesignSystem::RegisterAlertSignature(ETJAlarmSeverity S,const FString& Key,USoundBase* Cue){if(Key.IsEmpty()||!IsValid(Cue))return false;AlertCues.Add(MakeAlertKey(S,Key),Cue);return true;}
USoundBase* UTJAudioDesignSystem::ResolveAlertSignature(ETJAlarmSeverity S,const FString& Source)const{if(USoundBase* C=AlertCues.FindRef(MakeAlertKey(S,Source)))return C;return AlertCues.FindRef(MakeAlertKey(S,TEXT("*")));}
bool UTJAudioDesignSystem::TriggerAlarm(ETJAlarmSeverity S,const FString& Source){if(!World||Source.IsEmpty())return false;USoundBase* Cue=ResolveAlertSignature(S,Source);if(!Cue)return false;UGameplayStatics::PlaySound2D(World,Cue,1.f,1.f,0.f,nullptr,nullptr,false);return true;}
bool UTJAudioDesignSystem::BindTelemetryAlertSignature(ETJAlarmSeverity S,const FString& Key){return AlertCues.Contains(MakeAlertKey(S,Key))||AlertCues.Contains(MakeAlertKey(S,TEXT("*")));}
bool UTJAudioDesignSystem::BindServiceStateAudio(const FString& PackageId){
 if(!World||PackageId.IsEmpty())return false;
 for(TActorIterator<ATJServiceBuilding> It(World);It;++It)if(It->GetManifest().PackageId==PackageId){It->OnServiceStateTransitioned.AddDynamic(this,&UTJAudioDesignSystem::OnServiceTransition);return true;}
 return false;
}
void UTJAudioDesignSystem::OnServiceTransition(ETJServiceState From,ETJServiceState To){
 if(USoundBase* Cue=TransitionCues.FindRef(MakeTransitionKey(From,To)))for(TActorIterator<ATJServiceBuilding> It(World);It;++It)if(It->GetServiceState()==To){UGameplayStatics::SpawnSoundAtLocation(World,Cue,It->GetActorLocation(),FRotator::ZeroRotator,1.f,1.f,0.f,nullptr,nullptr,true);break;}
}
bool UTJAudioDesignSystem::ValidateServiceTransitionCueCoverage()const{
 const ETJServiceState S[]={ETJServiceState::Stopped,ETJServiceState::Starting,ETJServiceState::Running,ETJServiceState::Error,ETJServiceState::Updating};
 for(ETJServiceState A:S)for(ETJServiceState B:S)if(A!=B&&!TransitionCues.Contains(MakeTransitionKey(A,B)))return false;return true;
}