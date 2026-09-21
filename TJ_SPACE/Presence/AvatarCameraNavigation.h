#pragma once
#include "CoreMinimal.h"
#include "GameFramework/Pawn.h"
#include "Components/SceneComponent.h"
#include "AvatarCameraNavigation.generated.h"

UENUM(BlueprintType) enum class ETJCameraMode:uint8{FirstPerson,ThirdPerson,Orbit,VR};
UENUM(BlueprintType) enum class ETJLocomotionMode:uint8{Smooth,SnapTurn,Teleport,Seated};
USTRUCT(BlueprintType) struct TJSPACE_API FTJPresencePeer{GENERATED_BODY() UPROPERTY(BlueprintReadWrite) FString DeviceId; UPROPERTY(BlueprintReadWrite) FString DisplayName; UPROPERTY(BlueprintReadWrite) FTransform Transform; UPROPERTY(BlueprintReadWrite) bool bVR=false;};
USTRUCT(BlueprintType) struct TJSPACE_API FTJCameraBookmark{GENERATED_BODY() UPROPERTY(BlueprintReadWrite) FString Name; UPROPERTY(BlueprintReadWrite) FTransform Transform;};
USTRUCT(BlueprintType) struct TJSPACE_API FTJInteractionRequest{GENERATED_BODY() UPROPERTY(BlueprintReadWrite) FString ActorId; UPROPERTY(BlueprintReadWrite) FString InteractionId; UPROPERTY(BlueprintReadWrite) FString BackendRpc;};
UCLASS() class TJSPACE_API ATJOperatorAvatar:public APawn{GENERATED_BODY() public: ATJOperatorAvatar();};
UCLASS(BlueprintType) class TJSPACE_API UTJAvatarCameraNavigation:public UObject{GENERATED_BODY()
public:
 UFUNCTION(BlueprintCallable) bool SpawnOperatorAvatar(const FString&DeviceId);
 UFUNCTION(BlueprintCallable) bool SetCameraMode(ETJCameraMode Mode);
 UFUNCTION(BlueprintCallable) bool NavigateTo(const FString&TargetId);
 UFUNCTION(BlueprintCallable) bool EnterBuilding(const FString&PackageId);
 UFUNCTION(BlueprintCallable) bool InteractWithObject(const FString&Actor,const FString&Interaction);
 UFUNCTION(BlueprintCallable) bool RenderPresence(const TArray<FTJPresencePeer>&Peers);
 UFUNCTION(BlueprintCallable) bool SetCameraBookmark(const FString&Name,const FTransform&Transform);
 UFUNCTION(BlueprintCallable) bool EnableVRMode();
 UFUNCTION(BlueprintCallable) bool SetLocomotionMode(ETJLocomotionMode Mode);
 UFUNCTION(BlueprintCallable) bool ExecuteBackendRPC(const FString&Route,const FString&Payload);
private:
 UPROPERTY() TObjectPtr<ATJOperatorAvatar> Avatar;
 UPROPERTY() TMap<FString,FTJCameraBookmark> Bookmarks;
 UPROPERTY() TArray<FTJPresencePeer> Presence;
 ETJCameraMode CameraMode=ETJCameraMode::FirstPerson; ETJLocomotionMode LocomotionMode=ETJLocomotionMode::Smooth;
};