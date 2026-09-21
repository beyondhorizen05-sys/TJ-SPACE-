#pragma once
#include "CoreMinimal.h"
#include "GameFramework/Actor.h"
#include "Components/SceneComponent.h"
#include "Components/StaticMeshComponent.h"
#include "NetworkingTerrain.generated.h"

UENUM(BlueprintType)
enum class ETJNetworkStrategy : uint8 { LAN, Tor, Clearnet, WireGuard };

USTRUCT(BlueprintType)
struct TJSPACE_API FTJNetworkGateway { GENERATED_BODY()
 UPROPERTY(EditAnywhere,BlueprintReadWrite) FString GatewayId;
 UPROPERTY(EditAnywhere,BlueprintReadWrite) FString Hostname;
 UPROPERTY(EditAnywhere,BlueprintReadWrite) FVector Location=FVector::ZeroVector;
};

USTRUCT(BlueprintType)
struct TJSPACE_API FTJVPNPeer { GENERATED_BODY()
 UPROPERTY(EditAnywhere,BlueprintReadWrite) FString PeerId;
 UPROPERTY(EditAnywhere,BlueprintReadWrite) FString Endpoint;
 UPROPERTY(EditAnywhere,BlueprintReadWrite) FVector Location=FVector::ZeroVector;
 UPROPERTY(EditAnywhere,BlueprintReadWrite) bool bEnabled=true;
};

USTRUCT(BlueprintType)
struct TJSPACE_API FTJTLSCertificate { GENERATED_BODY()
 UPROPERTY(EditAnywhere,BlueprintReadWrite) FString Endpoint;
 UPROPERTY(EditAnywhere,BlueprintReadWrite) FString Subject;
 UPROPERTY(EditAnywhere,BlueprintReadWrite) FString Issuer;
 UPROPERTY(EditAnywhere,BlueprintReadWrite) FString Fingerprint;
 UPROPERTY(EditAnywhere,BlueprintReadWrite) double ValidUntilUnix=0.0;
};

USTRUCT(BlueprintType)
struct TJSPACE_API FTJDNSResolution { GENERATED_BODY()
 UPROPERTY(EditAnywhere,BlueprintReadWrite) FString Hostname;
 UPROPERTY(EditAnywhere,BlueprintReadWrite) TArray<FString> Addresses;
 UPROPERTY(EditAnywhere,BlueprintReadWrite) float Confidence01=1.0f;
};

USTRUCT(BlueprintType)
struct TJSPACE_API FTJNetworkStrategyConfig { GENERATED_BODY()
 UPROPERTY(EditAnywhere,BlueprintReadWrite) bool bLAN=true;
 UPROPERTY(EditAnywhere,BlueprintReadWrite) bool bTor=true;
 UPROPERTY(EditAnywhere,BlueprintReadWrite) bool bClearnet=true;
 UPROPERTY(EditAnywhere,BlueprintReadWrite) bool bWireGuard=true;
 UPROPERTY(EditAnywhere,BlueprintReadWrite) float ConstructionBlendSeconds=1.25f;
};

UCLASS()
class TJSPACE_API ATJNetworkingTerrain : public AActor {
 GENERATED_BODY()
public:
 ATJNetworkingTerrain();
 virtual void Tick(float DeltaSeconds) override;
 bool BuildLAN(); bool BuildTor(); bool BuildClearnet(const FTJNetworkGateway& Gateway); bool BuildVPN(const TArray<FTJVPNPeer>& Peers);
 bool RenderCertificate(const FTJTLSCertificate& Cert); bool RenderDNS(const FTJDNSResolution& Resolution); bool RenderTraffic();
 bool Configure(const FTJNetworkStrategyConfig& NewConfig);
 bool EnterTorLayer(bool bEnter);
private:
 UPROPERTY() TObjectPtr<USceneComponent> Root;
 UPROPERTY() TArray<TObjectPtr<UStaticMeshComponent>> LANStreets,TorTunnels,ClearnetBridges,VPNRoads,TLSShields,DNSBeams,TrafficPulses;
 FTJNetworkStrategyConfig Config; bool bTorEntered=false; float ConstructionAlpha=1.0f; float PulsePhase=0.0f;
 void SetScalar(UPrimitiveComponent* C,FName P,float V) const;
 void BuildSegment(TArray<TObjectPtr<UStaticMeshComponent>>& Target,const FVector& Loc,const FVector& Scale,FName Marker,float Value=1.0f);
};

UCLASS(BlueprintType)
class TJSPACE_API UTJNetworkingTerrain : public UObject {
 GENERATED_BODY()
public:
 UFUNCTION(BlueprintCallable) ATJNetworkingTerrain* BuildLANTerrain();
 UFUNCTION(BlueprintCallable) ATJNetworkingTerrain* BuildTorTunnels();
 UFUNCTION(BlueprintCallable) ATJNetworkingTerrain* BuildClearnetHighway(const FTJNetworkGateway& Gateway);
 UFUNCTION(BlueprintCallable) ATJNetworkingTerrain* BuildVPNRoads(const TArray<FTJVPNPeer>& Peers);
 UFUNCTION(BlueprintCallable) bool RenderTLSCertificate(const FTJTLSCertificate& Endpoint);
 UFUNCTION(BlueprintCallable) bool VisualizeDNSResolution(const FString& Hostname);
 UFUNCTION(BlueprintCallable) bool ShowTrafficFlow();
 UFUNCTION(BlueprintCallable) bool ConfigureNetworkStrategy(const FTJNetworkStrategyConfig& Strategy,const TArray<FTJVPNPeer>& Params);
 UFUNCTION(BlueprintCallable) bool EnterTorTunnelLayer(bool bEnter);
 UFUNCTION(BlueprintPure) ATJNetworkingTerrain* GetTerrain() const { return Terrain; }
 void RegisterDNS(const FTJDNSResolution& Resolution);
 void RegisterCertificate(const FTJTLSCertificate& Certificate);
private:
 UPROPERTY() TObjectPtr<ATJNetworkingTerrain> Terrain;
 UPROPERTY() TMap<FString,FTJDNSResolution> DNS;
 UPROPERTY() TMap<FString,FTJTLSCertificate> Certificates;
};