#pragma once
#include "CoreMinimal.h"
#include "BuildReleasePipeline.generated.h"
USTRUCT(BlueprintType) struct TJSPACE_API FTJReleaseArtifact { GENERATED_BODY()
 UPROPERTY(BlueprintReadOnly) FString Name; UPROPERTY(BlueprintReadOnly) FString Path; UPROPERTY(BlueprintReadOnly) FString Sha256; UPROPERTY(BlueprintReadOnly) FString Signature; UPROPERTY(BlueprintReadOnly) FString TargetTriple;
};
USTRUCT(BlueprintType) struct TJSPACE_API FTJRelease { GENERATED_BODY()
 UPROPERTY(BlueprintReadOnly) FString Version; UPROPERTY(BlueprintReadOnly) FString Channel; UPROPERTY(BlueprintReadOnly) TArray<FTJReleaseArtifact> Artifacts; UPROPERTY(BlueprintReadOnly) bool bSigned=false; UPROPERTY(BlueprintReadOnly) bool bPublished=false;
};
UCLASS(BlueprintType) class TJSPACE_API UTJBuildReleasePipeline:public UObject { GENERATED_BODY()
public:
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Release") bool BuildRustBinaries(const FString& TargetTriple);
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Release") bool BuildWebUIs(const FString& AppId);
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Release") bool BuildSdk();
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Release") bool BuildInstallerImage(const FString& TargetTriple);
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Release") bool PackageRelease(const FString& Version,const FString& Channel);
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Release") bool SignRelease(const FTJReleaseArtifact& Artifact,const FString& ReleaseKey);
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Release") bool PublishRelease(const FTJRelease& Release);
 UFUNCTION(BlueprintCallable,Category="TJ SPACE|Release") bool VerifyRelease(const FTJReleaseArtifact& Artifact) const;
 UFUNCTION(BlueprintPure,Category="TJ SPACE|Release") TArray<FString> GetSupportedTargets() const;
 UFUNCTION(BlueprintPure,Category="TJ SPACE|Release") FTJRelease GetCurrentRelease() const{return CurrentRelease;}
private:
 bool IsSupportedTarget(const FString& T) const;
 UPROPERTY() FTJRelease CurrentRelease;
};