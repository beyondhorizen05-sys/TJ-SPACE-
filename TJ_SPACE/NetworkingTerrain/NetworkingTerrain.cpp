#include "NetworkingTerrain.h"
#include "Engine/World.h"
#include "Materials/MaterialInstanceDynamic.h"

ATJNetworkingTerrain::ATJNetworkingTerrain(){ PrimaryActorTick.bCanEverTick=true; Root=CreateDefaultSubobject<USceneComponent>(TEXT("Root")); SetRootComponent(Root); }
void ATJNetworkingTerrain::SetScalar(UPrimitiveComponent* C,FName P,float V) const{ if(!IsValid(C)) return; for(int32 i=0;i<C->GetNumMaterials();++i) if(auto* M=C->CreateAndSetMaterialInstanceDynamic(i)) M->SetScalarParameterValue(P,V); }
void ATJNetworkingTerrain::BuildSegment(TArray<TObjectPtr<UStaticMeshComponent>>& Target,const FVector& Loc,const FVector& Scale,FName Marker,float Value){
 auto* C=NewObject<UStaticMeshComponent>(this); C->RegisterComponent(); C->AttachToComponent(Root,FAttachmentTransformRules::KeepRelativeTransform); C->SetRelativeLocation(Loc); C->SetRelativeScale3D(Scale); C->SetCollisionEnabled(ECollisionEnabled::NoCollision); Target.Add(C); SetScalar(C,Marker,Value); SetScalar(C,TEXT("ConstructionBlend"),ConstructionAlpha);
}
bool ATJNetworkingTerrain::BuildLAN(){ LANStreets.Reset(); for(int32 i=0;i<8;++i) BuildSegment(LANStreets,FVector(i*900,0,0),FVector(4.0f,.18f,.12f),TEXT("LANStreet01")); return true; }
bool ATJNetworkingTerrain::BuildTor(){ TorTunnels.Reset(); for(int32 i=0;i<6;++i) BuildSegment(TorTunnels,FVector(i*850,0,-420),FVector(2.8f,.65f,.65f),TEXT("TorTunnel01")); return true; }
bool ATJNetworkingTerrain::BuildClearnet(const FTJNetworkGateway& Gateway){ ClearnetBridges.Reset(); BuildSegment(ClearnetBridges,Gateway.Location,FVector(5.0f,.22f,.15f),TEXT("ClearnetGateway01")); return !Gateway.GatewayId.IsEmpty(); }
bool ATJNetworkingTerrain::BuildVPN(const TArray<FTJVPNPeer>& Peers){ VPNRoads.Reset(); for(int32 i=0;i<Peers.Num();++i) if(Peers[i].bEnabled) BuildSegment(VPNRoads,Peers[i].Location,FVector(2.0f,.28f,.18f),TEXT("WireGuardRoad01")); return true; }
bool ATJNetworkingTerrain::RenderCertificate(const FTJTLSCertificate& Cert){ BuildSegment(TLSShields,FVector(TLSShields.Num()*140,0,260),FVector(.35f,.35f,.55f),TEXT("TLSShield01"),Cert.Endpoint.IsEmpty()?0.0f:1.0f); return !Cert.Endpoint.IsEmpty(); }
bool ATJNetworkingTerrain::RenderDNS(const FTJDNSResolution& Resolution){ BuildSegment(DNSBeams,FVector(DNSBeams.Num()*120,0,320),FVector(.18f,.18f,2.2f),TEXT("DNSBeam01"),FMath::Clamp(Resolution.Confidence01,0.0f,1.0f)); return !Resolution.Hostname.IsEmpty(); }
bool ATJNetworkingTerrain::RenderTraffic(){ if(LANStreets.Num()==0&&TorTunnels.Num()==0&&ClearnetBridges.Num()==0&&VPNRoads.Num()==0) return false; TrafficPulses.Reset(); for(int32 i=0;i<12;++i) BuildSegment(TrafficPulses,FVector(i*520,0,80),FVector(.08f,.08f,.08f),TEXT("TrafficPulse01")); return true; }
void ATJNetworkingTerrain::Tick(float DeltaSeconds){ Super::Tick(DeltaSeconds); PulsePhase+=FMath::Max(0.f,DeltaSeconds); for(int32 i=0;i<TrafficPulses.Num();++i) if(IsValid(TrafficPulses[i])) SetScalar(TrafficPulses[i],TEXT("TrafficPulsePhase"),FMath::Frac(PulsePhase*.8f+i*.0833f)); }
bool ATJNetworkingTerrain::Configure(const FTJNetworkStrategyConfig& NewConfig,const TArray<FTJVPNPeer>& Peers){ Config=NewConfig; ConstructionAlpha=1.0f; BuildLAN(); BuildTor(); FTJNetworkGateway G; G.GatewayId=TEXT("default"); G.Hostname=TEXT("gateway"); G.Location=FVector(7200,0,0); BuildClearnet(G); BuildVPN(Peers); return true; }
bool ATJNetworkingTerrain::EnterTorLayer(bool bEnter){ bTorEntered=bEnter; SetScalar(nullptr,TEXT("TorLighting01"),0); for(auto* C:TorTunnels) if(IsValid(C)){ SetScalar(C,TEXT("TorLighting01"),bEnter?1.f:0.f); SetScalar(C,TEXT("TorFog01"),bEnter?1.f:0.f); SetScalar(C,TEXT("TorAudioOcclusion01"),bEnter?1.f:0.f); } return true; }

ATJNetworkingTerrain* UTJNetworkingTerrain::BuildLANTerrain(){ if(!Terrain){ UWorld* W=GetWorld(); if(!IsValid(W)) return nullptr; Terrain=W->SpawnActor<ATJNetworkingTerrain>(); } return Terrain->BuildLAN()?Terrain:nullptr; }
ATJNetworkingTerrain* UTJNetworkingTerrain::BuildTorTunnels(){ if(!Terrain){ if(!BuildLANTerrain()) return nullptr; } return Terrain->BuildTor()?Terrain:nullptr; }
ATJNetworkingTerrain* UTJNetworkingTerrain::BuildClearnetHighway(const FTJNetworkGateway& Gateway){ if(!Terrain&& !BuildLANTerrain()) return nullptr; return Terrain->BuildClearnet(Gateway)?Terrain:nullptr; }
ATJNetworkingTerrain* UTJNetworkingTerrain::BuildVPNRoads(const TArray<FTJVPNPeer>& Peers){ if(!Terrain&& !BuildLANTerrain()) return nullptr; return Terrain->BuildVPN(Peers)?Terrain:nullptr; }
bool UTJNetworkingTerrain::RenderTLSCertificate(const FTJTLSCertificate& Endpoint){ if(!Terrain&& !BuildLANTerrain()) return false; RegisterCertificate(Endpoint); return Terrain->RenderCertificate(Endpoint); }
bool UTJNetworkingTerrain::VisualizeDNSResolution(const FString& Hostname){ if(!Terrain&& !BuildLANTerrain()) return false; const FTJDNSResolution* R=DNS.Find(Hostname); return R?Terrain->RenderDNS(*R):false; }
bool UTJNetworkingTerrain::ShowTrafficFlow(){ return IsValid(Terrain)&&Terrain->RenderTraffic(); }
bool UTJNetworkingTerrain::ConfigureNetworkStrategy(const FTJNetworkStrategyConfig& Strategy,const TArray<FTJVPNPeer>& Params){ if(!Terrain&& !BuildLANTerrain()) return false; if(Strategy.bLAN) Terrain->BuildLAN(); if(Strategy.bTor) Terrain->BuildTor(); if(Strategy.bClearnet){FTJNetworkGateway G;G.GatewayId=TEXT("configured");Terrain->BuildClearnet(G);} if(Strategy.bWireGuard) Terrain->BuildVPN(Params); return true; }
bool UTJNetworkingTerrain::EnterTorTunnelLayer(bool bEnter){ return IsValid(Terrain)&&Terrain->EnterTorLayer(bEnter); }
void UTJNetworkingTerrain::RegisterDNS(const FTJDNSResolution& R){ if(!R.Hostname.IsEmpty()) DNS.Add(R.Hostname,R); }
void UTJNetworkingTerrain::RegisterCertificate(const FTJTLSCertificate& C){ if(!C.Endpoint.IsEmpty()) Certificates.Add(C.Endpoint,C); }
