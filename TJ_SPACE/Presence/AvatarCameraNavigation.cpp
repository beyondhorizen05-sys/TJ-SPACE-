#include "AvatarCameraNavigation.h"
#include "Engine/World.h"
ATJOperatorAvatar::ATJOperatorAvatar(){PrimaryActorTick.bCanEverTick=false;RootComponent=CreateDefaultSubobject<USceneComponent>(TEXT("Root"));AutoPossessPlayer=EAutoReceiveInput::Player0;}
bool UTJAvatarCameraNavigation::SpawnOperatorAvatar(const FString&DeviceId){if(DeviceId.IsEmpty()||!GetWorld())return false;if(!Avatar){Avatar=GetWorld()->SpawnActor<ATJOperatorAvatar>();}Avatar->Tags.AddUnique(FName(*FString::Printf(TEXT("device:%s"),*DeviceId)));return Avatar!=nullptr;}
bool UTJAvatarCameraNavigation::SetCameraMode(ETJCameraMode Mode){CameraMode=Mode;return true;}
bool UTJAvatarCameraNavigation::NavigateTo(const FString&TargetId){if(TargetId.IsEmpty())return false;return ExecuteBackendRPC(TEXT("/api/v1/navigation/navigate"),TargetId);}
bool UTJAvatarCameraNavigation::EnterBuilding(const FString&PackageId){if(PackageId.IsEmpty())return false;return ExecuteBackendRPC(TEXT("/api/v1/services/enter"),PackageId);}
bool UTJAvatarCameraNavigation::InteractWithObject(const FString&Actor,const FString&Interaction){if(Actor.IsEmpty()||Interaction.IsEmpty())return false;const FString Route=FString::Printf(TEXT("/api/v1/interactions/%s"),*Interaction);return ExecuteBackendRPC(Route,Actor);}
bool UTJAvatarCameraNavigation::RenderPresence(const TArray<FTJPresencePeer>&Peers){Presence=Peers;return true;}
bool UTJAvatarCameraNavigation::SetCameraBookmark(const FString&Name,const FTransform&Transform){if(Name.IsEmpty())return false;Bookmarks.Add(Name,{Name,Transform});return true;}
bool UTJAvatarCameraNavigation::EnableVRMode(){CameraMode=ETJCameraMode::VR;LocomotionMode=ETJLocomotionMode::Teleport;return ExecuteBackendRPC(TEXT("/api/v1/presence/vr"),TEXT("enable"));}
bool UTJAvatarCameraNavigation::SetLocomotionMode(ETJLocomotionMode Mode){LocomotionMode=Mode;return true;}
bool UTJAvatarCameraNavigation::ExecuteBackendRPC(const FString&Route,const FString&Payload){return Route.StartsWith(TEXT("/api/v1/"))&&!Payload.IsEmpty();}