#pragma once
#include "CoreMinimal.h"
#include "GameFramework/Actor.h"
#include "Components/SceneComponent.h"
#include "Components/StaticMeshComponent.h"
#include "IdentityAuthDevices.generated.h"

UENUM(BlueprintType) enum class ETJDeviceState:uint8{ Active, Revoked };
UENUM(BlueprintType) enum class ETJDoorAuthState:uint8{ Idle, Authorizing, Authorized, Denied };
USTRUCT(BlueprintType) struct TJSPACE_API FTJDeviceIdentity{GENERATED_BODY()
 UPROPERTY(EditAnywhere,BlueprintReadWrite) FString DeviceId; UPROPERTY(EditAnywhere,BlueprintReadWrite) FString DisplayName; UPROPERTY(EditAnywhere,BlueprintReadWrite) FString ExpectedSignature; UPROPERTY(EditAnywhere,BlueprintReadWrite) ETJDeviceState State=ETJDeviceState::Active;};
USTRUCT(BlueprintType) struct TJSPACE_API FTJKeycard{GENERATED_BODY()
 UPROPERTY(EditAnywhere,BlueprintReadWrite) FString DeviceId; UPROPERTY(EditAnywhere,BlueprintReadWrite) TArray<FString> Permissions;};
USTRUCT(BlueprintType) struct TJSPACE_API FTJLocalAuthSession{GENERATED_BODY()
 UPROPERTY(EditAnywhere,BlueprintReadWrite) FString SessionId; UPROPERTY(EditAnywhere,BlueprintReadWrite) FString DeviceId; UPROPERTY(EditAnywhere,BlueprintReadWrite) double IssuedAtSeconds=0; UPROPERTY(EditAnywhere,BlueprintReadWrite) double ExpiresAtSeconds=0; UPROPERTY(EditAnywhere,BlueprintReadWrite) bool bRevoked=false;};
USTRUCT(BlueprintType) struct TJSPACE_API FTJAccessLedgerEntry{GENERATED_BODY()
 UPROPERTY(EditAnywhere,BlueprintReadWrite) double TimestampSeconds=0; UPROPERTY(EditAnywhere,BlueprintReadWrite) FString DeviceId; UPROPERTY(EditAnywhere,BlueprintReadWrite) bool bAuthorized=false; UPROPERTY(EditAnywhere,BlueprintReadWrite) FString Reason;};
UCLASS() class TJSPACE_API ATJIdentityGatehouse:public AActor{GENERATED_BODY()
public: ATJIdentityGatehouse(); virtual void Tick(float DeltaSeconds) override;
bool RenderDeviceAvatar(const FString&DeviceId); bool RenderKeycard(const FString&DeviceId,const TArray<FString>&Permissions); bool AuthenticateDevice(const FString&DeviceId,const FString&Signature); bool RenderLocalAuthCookie(const FTJLocalAuthSession&Session); bool ManageDeviceFleet(); bool RenderAccessLog(); bool RegisterDevice(const FTJDeviceIdentity&Device); bool RevokeDevice(const FString&DeviceId);
private:
UPROPERTY() TObjectPtr<USceneComponent> Root; UPROPERTY() TObjectPtr<UStaticMeshComponent> Gate; UPROPERTY() TObjectPtr<UStaticMeshComponent> AuthBeam; UPROPERTY() TObjectPtr<UStaticMeshComponent> DeniedPlacard; UPROPERTY() TObjectPtr<UStaticMeshComponent> Ledger; UPROPERTY() TObjectPtr<UStaticMeshComponent> FleetControl; UPROPERTY() TMap<FString,FTJDeviceIdentity> Devices; UPROPERTY() TArray<TObjectPtr<UStaticMeshComponent>> Avatars; UPROPERTY() TArray<TObjectPtr<UStaticMeshComponent>> Keycards; UPROPERTY() TArray<TObjectPtr<UStaticMeshComponent>> Passes; UPROPERTY() TArray<FTJAccessLedgerEntry> AccessHistory; double RuntimeSeconds=0;
void SetScalar(UPrimitiveComponent*C,FName P,float V) const; void SetDoor(ETJDoorAuthState State);};
UCLASS(BlueprintType) class TJSPACE_API UTJIdentityAuthDevices:public UObject{GENERATED_BODY()
public:
UFUNCTION(BlueprintCallable) ATJIdentityGatehouse* BuildGatehouse();
UFUNCTION(BlueprintCallable) bool RenderDeviceAvatar(const FString&DeviceId); UFUNCTION(BlueprintCallable) bool RenderKeycard(const FString&DeviceId,const TArray<FString>&Permissions); UFUNCTION(BlueprintCallable) bool AuthenticateDevice(const FString&DeviceId,const FString&Signature); UFUNCTION(BlueprintCallable) bool RenderLocalAuthCookie(const FTJLocalAuthSession&Session); UFUNCTION(BlueprintCallable) bool ManageDeviceFleet(); UFUNCTION(BlueprintCallable) bool RenderAccessLog();
bool RegisterDevice(const FTJDeviceIdentity&Device); bool RevokeDevice(const FString&DeviceId);
private: UPROPERTY() TObjectPtr<ATJIdentityGatehouse> Gatehouse;};