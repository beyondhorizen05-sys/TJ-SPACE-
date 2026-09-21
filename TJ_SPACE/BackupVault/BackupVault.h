#pragma once
#include "CoreMinimal.h"
#include "GameFramework/Actor.h"
#include "Components/SceneComponent.h"
#include "Components/StaticMeshComponent.h"
#include "BackupVault.generated.h"

UENUM(BlueprintType) enum class ETJBackupState:uint8{ Creating, Current, Superseded, Restoring, Verified, Corrupt, Deleted };
USTRUCT(BlueprintType) struct TJSPACE_API FTJBackupTarget{GENERATED_BODY() UPROPERTY(EditAnywhere,BlueprintReadWrite) FString TargetId; UPROPERTY(EditAnywhere,BlueprintReadWrite) FString DisplayName;};
USTRUCT(BlueprintType) struct TJSPACE_API FTJBackupMonolith{GENERATED_BODY() UPROPERTY(EditAnywhere,BlueprintReadWrite) FString MonolithId; UPROPERTY(EditAnywhere,BlueprintReadWrite) FString PackageId; UPROPERTY(EditAnywhere,BlueprintReadWrite) FString TargetId; UPROPERTY(EditAnywhere,BlueprintReadWrite) FString Version; UPROPERTY(EditAnywhere,BlueprintReadWrite) ETJBackupState State=ETJBackupState::Creating; UPROPERTY(EditAnywhere,BlueprintReadWrite) float Progress01=0; UPROPERTY(EditAnywhere,BlueprintReadWrite) FString IntegrityDigest;};
USTRUCT(BlueprintType) struct TJSPACE_API FTJBackupSchedule{GENERATED_BODY() UPROPERTY(EditAnywhere,BlueprintReadWrite) FString ScheduleId; UPROPERTY(EditAnywhere,BlueprintReadWrite) FString PackageId; UPROPERTY(EditAnywhere,BlueprintReadWrite) FString Cron; UPROPERTY(EditAnywhere,BlueprintReadWrite) bool bEnabled=true;};
USTRUCT(BlueprintType) struct TJSPACE_API FTJRestoreInterlock{GENERATED_BODY() UPROPERTY(EditAnywhere,BlueprintReadWrite) FString MonolithId; UPROPERTY(EditAnywhere,BlueprintReadWrite) float HoldSeconds=0; UPROPERTY(EditAnywhere,BlueprintReadWrite) bool bEngaged=false;};
UCLASS() class TJSPACE_API ATJBackupVault:public AActor{GENERATED_BODY()
public: ATJBackupVault(); virtual void Tick(float DeltaSeconds) override;
bool Build(); bool RenderTarget(const FString& Id); bool Create(const FString& PackageId,const FString& TargetId); bool Progress(const FString& PackageId,float Value01); bool Restore(const FString& MonolithId,const FString& TargetService); bool Verify(const FString& MonolithId); bool Delete(const FString& MonolithId); bool RenderSchedule();
bool HoldRestoreInterlock(const FString& MonolithId,float DeltaSeconds); bool ReleaseRestoreInterlock(const FString& MonolithId);
void RegisterTarget(const FTJBackupTarget& T); void RegisterSchedule(const FTJBackupSchedule& S);
private: UPROPERTY() TObjectPtr<USceneComponent> Root; UPROPERTY() TObjectPtr<UStaticMeshComponent> VaultWing; UPROPERTY() TObjectPtr<UStaticMeshComponent> CalendarObelisk; UPROPERTY() TArray<TObjectPtr<UStaticMeshComponent>> Chambers; UPROPERTY() TArray<TObjectPtr<UStaticMeshComponent>> Monoliths; UPROPERTY() TArray<TObjectPtr<UStaticMeshComponent>> DissolutionEffects; UPROPERTY() TArray<TObjectPtr<UStaticMeshComponent>> RestoreDischarges; UPROPERTY() TArray<TObjectPtr<UStaticMeshComponent>> IntegritySeals; UPROPERTY() TArray<TObjectPtr<UStaticMeshComponent>> DeleteWarnings; UPROPERTY() TArray<TObjectPtr<UStaticMeshComponent>> Interlocks;
TMap<FString,FTJBackupTarget> Targets; TMap<FString,FTJBackupMonolith> Backups; TMap<FString,FTJBackupSchedule> Schedules; TMap<FString,FTJRestoreInterlock> InterlocksState;
void SetScalar(UPrimitiveComponent* C,FName P,float V) const; void RenderDissolution(const FString& TargetId);};
UCLASS(BlueprintType) class TJSPACE_API UTJBackupVault:public UObject{GENERATED_BODY()
public:
UFUNCTION(BlueprintCallable) ATJBackupVault* BuildBackupVault(); UFUNCTION(BlueprintCallable) bool RenderBackupTarget(const FString& TargetId); UFUNCTION(BlueprintCallable) bool CreateBackup(const FString& PackageId,const FString& TargetId); UFUNCTION(BlueprintCallable) bool RenderBackupProgress(const FString& PackageId,float Progress); UFUNCTION(BlueprintCallable) bool RestoreBackup(const FString& MonolithId,const FString& TargetService); UFUNCTION(BlueprintCallable) bool VerifyBackupIntegrity(const FString& MonolithId); UFUNCTION(BlueprintCallable) bool DeleteBackup(const FString& MonolithId); UFUNCTION(BlueprintCallable) bool RenderBackupSchedule(); UFUNCTION(BlueprintCallable) bool HoldRestoreInterlock(const FString& MonolithId,float DeltaSeconds); UFUNCTION(BlueprintCallable) bool ReleaseRestoreInterlock(const FString& MonolithId);
void RegisterTarget(const FTJBackupTarget& T); void RegisterSchedule(const FTJBackupSchedule& S);
private: UPROPERTY() TObjectPtr<ATJBackupVault> Vault;};