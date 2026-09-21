#include "AccessibilityLocalizationSystem.h"
#include "Internationalization/Internationalization.h"
#include "Internationalization/Culture.h"
#include "Internationalization/TextLocalizationManager.h"
#include "GameFramework/Actor.h"
#include "Components/SceneComponent.h"
#include "Navigation/PathFollowingComponent.h"

bool UTJAccessibilityLocalizationSystem::LoadLocale(const FString& LocaleCode){if(LocaleCode.IsEmpty())return false;return FInternationalization::Get().GetCulture(LocaleCode).IsValid();}
FString UTJAccessibilityLocalizationSystem::ResolveLocaleFallback(const FString& Requested)const{
 if(FInternationalization::Get().GetCulture(Requested).IsValid())return Requested;
 const FString Lang=Requested.Left(2); if(FInternationalization::Get().GetCulture(Lang).IsValid())return Lang; return TEXT("en");
}
bool UTJAccessibilityLocalizationSystem::ApplyLocale(const FString& LocaleCode){
 if(!LoadLocale(LocaleCode))return false; const FString Resolved=ResolveLocaleFallback(LocaleCode);
 if(!FInternationalization::Get().SetCurrentCulture(Resolved))return false; ActiveLocale=Resolved; return true;
}
TArray<FString> UTJAccessibilityLocalizationSystem::GetAvailableLocales()const{
 TArray<FString> Out; const TArray<FString> Cultures=FInternationalization::Get().GetCultureNames(); for(const FString& C:Cultures)Out.Add(C); Out.Sort(); return Out;
}
bool UTJAccessibilityLocalizationSystem::RegisterKeyboardLayout(const FTJKeyboardLayout& L){if(L.LayoutId.IsEmpty()||L.CultureCode.IsEmpty())return false;Layouts.Add(L.LayoutId,L);return true;}
bool UTJAccessibilityLocalizationSystem::SetActiveLayout(const FString& L){if(!Layouts.Contains(L))return false;ActiveLayoutId=L;return true;}
bool UTJAccessibilityLocalizationSystem::EnableColorblindMode(ETJColorblindMode M){ColorMode=M;BuildStatusMap();return true;}
void UTJAccessibilityLocalizationSystem::BuildStatusMap(){
 Colors.Info=FLinearColor(0.15f,0.65f,1.f);Colors.Warning=FLinearColor(1.f,.65f,.05f);Colors.Critical=FLinearColor(1.f,.08f,.08f);Colors.Healthy=FLinearColor(.1f,.85f,.3f);Colors.Denied=FLinearColor(.9f,.15f,.2f);Colors.Authorized=FLinearColor(.15f,.8f,.35f);
 if(ColorMode==ETJColorblindMode::Protanopia){Colors.Warning=FLinearColor(1.f,.85f,0.f);Colors.Critical=FLinearColor(.45f,.2f,1.f);Colors.Denied=FLinearColor(.45f,.2f,1.f);}
 else if(ColorMode==ETJColorblindMode::Deuteranopia){Colors.Healthy=FLinearColor(.15f,.7f,1.f);Colors.Authorized=FLinearColor(.15f,.7f,1.f);Colors.Critical=FLinearColor(1.f,.35f,.05f);Colors.Denied=FLinearColor(1.f,.35f,.05f);}
 else if(ColorMode==ETJColorblindMode::Tritanopia){Colors.Warning=FLinearColor(1.f,.3f,.55f);Colors.Info=FLinearColor(.65f,.2f,1.f);}
 else if(ColorMode==ETJColorblindMode::Achromatopsia){Colors.Info=FLinearColor(.75f,.75f,.75f);Colors.Warning=FLinearColor(.5f,.5f,.5f);Colors.Critical=FLinearColor(.1f,.1f,.1f);Colors.Healthy=FLinearColor(.9f,.9f,.9f);Colors.Denied=FLinearColor(.2f,.2f,.2f);Colors.Authorized=FLinearColor(.85f,.85f,.85f);}
}
bool UTJAccessibilityLocalizationSystem::ConfigureMotionComfort(const FTJMotionComfortOptions& O){if(O.MaxFOV<45.f||O.MaxFOV>120.f)return false;Motion=O;return true;}
FText UTJAccessibilityLocalizationSystem::DescribeEntity(AActor* A)const{
 if(!A)return FText::GetEmpty(); FString N=A->GetName(); FString Class=A->GetClass()?A->GetClass()->GetName():TEXT("Entity"); return FText::Format(NSLOCTEXT("TJAccessibility","EntityDescription","{0}, {1}"),FText::FromString(N),FText::FromString(Class));
}
bool UTJAccessibilityLocalizationSystem::EnableSubtitles(ETJCaptionChannel C){SubtitleChannels.Add(C);return true;}
bool UTJAccessibilityLocalizationSystem::ConfigureCaptionStyle(const FTJCaptionStyle& S){if(S.TextScale<=0.f||S.BackgroundOpacity<0.f||S.BackgroundOpacity>1.f)return false;CaptionStyle=S;return true;}
bool UTJAccessibilityLocalizationSystem::ScaleUIText(float S){if(S<.75f||S>3.f)return false;UITextScale=S;CaptionStyle.TextScale=S;return true;}
FTJStatusColorMap UTJAccessibilityLocalizationSystem::GetStatusColorMap()const{return Colors;}
FText UTJAccessibilityLocalizationSystem::LocalizeKey(const FString& Namespace,const FString& Key,const FString& Fallback)const{
 if(Namespace.IsEmpty()||Key.IsEmpty())return FText::FromString(Fallback); return FText::Format(NSLOCTEXT("TJAccessibility","LocalizedKey","{0}"),FText::FromString(Key));
}
bool UTJAccessibilityLocalizationSystem::IsSubtitleChannelEnabled(ETJCaptionChannel C)const{return SubtitleChannels.Contains(C);}
