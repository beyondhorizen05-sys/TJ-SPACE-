#include "ContainerIsolationVisualization.h"
#include "Engine/World.h"
#include "Materials/MaterialInstanceDynamic.h"

ATJContainerPod::ATJContainerPod(){
    PrimaryActorTick.bCanEverTick=true;
    Root=CreateDefaultSubobject<USceneComponent>(TEXT("Root")); SetRootComponent(Root);
    PodShell=CreateDefaultSubobject<UStaticMeshComponent>(TEXT("PodShell"));
    PodShell->SetupAttachment(Root);
    PodShell->SetCollisionEnabled(ECollisionEnabled::QueryAndPhysics);
}
void ATJContainerPod::Configure(const FTJContainerManifest& InManifest){
    Manifest=InManifest; Health=Manifest.Health;
}
void ATJContainerPod::SetScalar(UPrimitiveComponent* C,FName P,float V) const{
    if(!IsValid(C)) return;
    for(int32 i=0;i<C->GetNumMaterials();++i)
        if(UMaterialInstanceDynamic* M=C->CreateAndSetMaterialInstanceDynamic(i)) M->SetScalarParameterValue(P,V);
}
bool ATJContainerPod::RenderBoundary(ETJContainerBoundaryMode Mode){
    BoundaryMode=Mode; BoundaryLayers.Reset();
    const int32 Count=(Mode==ETJContainerBoundaryMode::Solid)?1:3;
    for(int32 i=0;i<Count;++i){
        auto* Layer=NewObject<UStaticMeshComponent>(this); Layer->RegisterComponent();
        Layer->AttachToComponent(Root,FAttachmentTransformRules::KeepRelativeTransform);
        Layer->SetRelativeScale3D(FVector(1.0f+0.06f*i));
        Layer->SetCollisionEnabled(ECollisionEnabled::NoCollision);
        BoundaryLayers.Add(Layer);
        SetScalar(Layer,TEXT("IsolationXRay01"),Mode==ETJContainerBoundaryMode::XRay||Mode==ETJContainerBoundaryMode::Isolation?1.0f:0.0f);
        SetScalar(Layer,TEXT("TrafficGate01"),Mode==ETJContainerBoundaryMode::Traffic?1.0f:0.0f);
    }
    return true;
}
bool ATJContainerPod::RenderSubcontainers(){
    Subcontainers.Reset();
    for(int32 i=0;i<Manifest.SubcontainerIds.Num();++i){
        auto* S=NewObject<UStaticMeshComponent>(this); S->RegisterComponent();
        S->AttachToComponent(Root,FAttachmentTransformRules::KeepRelativeTransform);
        const float A=2.0f*PI*i/FMath::Max(1,Manifest.SubcontainerIds.Num());
        S->SetRelativeLocation(FVector(FMath::Cos(A)*180,FMath::Sin(A)*180,0));
        S->SetRelativeScale3D(FVector(0.42f)); S->SetCollisionEnabled(ECollisionEnabled::NoCollision);
        Subcontainers.Add(S); SetScalar(S,TEXT("IsolationXRay01"),1.0f);
    }
    return true;
}
bool ATJContainerPod::RenderVolumes(){
    VolumeMounts.Reset();
    for(int32 i=0;i<Manifest.VolumeMounts.Num();++i){
        auto* V=NewObject<UStaticMeshComponent>(this); V->RegisterComponent();
        V->AttachToComponent(Root,FAttachmentTransformRules::KeepRelativeTransform);
        V->SetRelativeLocation(FVector(0,0,120+i*35)); V->SetRelativeScale3D(FVector(0.8f,0.12f,0.12f));
        V->SetCollisionEnabled(ECollisionEnabled::NoCollision); VolumeMounts.Add(V);
        SetScalar(V,TEXT("VolumeReadOnly01"),Manifest.VolumeMounts[i].bReadOnly?1.0f:0.0f);
        SetScalar(V,TEXT("VolumeMounted01"),1.0f);
    }
    return true;
}
bool ATJContainerPod::RenderLimits(){
    ResourceGovernors.Reset();
    const float Values[4]={Manifest.Limits.CpuLimit01,Manifest.Limits.MemoryLimit01,Manifest.Limits.PidsLimit01,Manifest.Limits.IoLimit01};
    for(int32 i=0;i<4;++i){
        auto* G=NewObject<UStaticMeshComponent>(this); G->RegisterComponent();
        G->AttachToComponent(Root,FAttachmentTransformRules::KeepRelativeTransform);
        G->SetRelativeLocation(FVector(-120+i*80,0,-140)); G->SetRelativeScale3D(FVector(0.25f,0.25f,FMath::Clamp(Values[i],0.02f,1.0f)));
        G->SetCollisionEnabled(ECollisionEnabled::NoCollision); ResourceGovernors.Add(G);
        SetScalar(G,TEXT("GovernorLimit01"),FMath::Clamp(Values[i],0.0f,1.0f));
    }
    return true;
}
bool ATJContainerPod::RenderHealth(){
    VitalSigns.Reset();
    auto* V=NewObject<UStaticMeshComponent>(this); V->RegisterComponent();
    V->AttachToComponent(Root,FAttachmentTransformRules::KeepRelativeTransform);
    V->SetRelativeLocation(FVector(0,0,170)); V->SetRelativeScale3D(FVector(0.18f));
    V->SetCollisionEnabled(ECollisionEnabled::NoCollision); VitalSigns.Add(V);
    const float H=Health==ETJContainerHealth::Healthy?1.0f:Health==ETJContainerHealth::Degraded?0.65f:Health==ETJContainerHealth::Critical?0.25f:Health==ETJContainerHealth::Stopped?0.0f:0.5f;
    SetScalar(V,TEXT("VitalHealth01"),H); return true;
}
void ATJContainerPod::SetHealth(ETJContainerHealth NewHealth){Health=NewHealth; RenderHealth();}
void ATJContainerPod::Tick(float DeltaSeconds){
    Super::Tick(DeltaSeconds); VitalPhase+=FMath::Max(DeltaSeconds,0.0f);
    const float H=Health==ETJContainerHealth::Healthy?1.0f:Health==ETJContainerHealth::Degraded?0.65f:Health==ETJContainerHealth::Critical?0.25f:0.0f;
    const float Rate=Health==ETJContainerHealth::Critical?3.5f:Health==ETJContainerHealth::Degraded?1.8f:1.0f;
    for(auto* V:VitalSigns) if(IsValid(V)) SetScalar(V,TEXT("VitalPulse01"),0.5f+0.5f*FMath::Sin(VitalPhase*Rate)*H);
}
void ATJContainerPod::Archive(){SetActorHiddenInGame(true);SetActorEnableCollision(false);SetActorTickEnabled(false);}
void UTJContainerIsolationVisualization::RegisterManifest(const FTJContainerManifest& Manifest){if(!Manifest.ContainerId.IsEmpty()) Manifests.Add(Manifest.ContainerId,Manifest);}
ATJContainerPod* UTJContainerIsolationVisualization::RenderContainerPod(const FString& ContainerId){
    if(ContainerId.IsEmpty()) return nullptr; if(auto* Existing=ResolveContainer(ContainerId)) return Existing;
    UWorld* W=GetWorld(); if(!IsValid(W)||!Manifests.Contains(ContainerId)) return nullptr;
    auto* Pod=W->SpawnActor<ATJContainerPod>(ATJContainerPod::StaticClass(),FTransform::Identity); if(!IsValid(Pod)) return nullptr;
    Pod->Configure(Manifests[ContainerId]); Pod->RenderBoundary(ETJContainerBoundaryMode::Solid);
    Pods.Add(ContainerId,Pod); return Pod;
}
ATJContainerPod* UTJContainerIsolationVisualization::ResolveContainer(const FString& ContainerId) const{return Pods.FindRef(ContainerId);}
bool UTJContainerIsolationVisualization::ShowContainerBoundary(const FString& Id,ETJContainerBoundaryMode Mode){auto* P=ResolveContainer(Id); return IsValid(P)&&P->RenderBoundary(Mode);}
bool UTJContainerIsolationVisualization::RenderSubcontainers(const FString& PackageId){
    bool Any=false; for(auto& Pair:Manifests) if(Pair.Value.PackageId==PackageId){auto* P=ResolveContainer(Pair.Key); if(IsValid(P)) Any=P->RenderSubcontainers()||Any;} return Any;
}
bool UTJContainerIsolationVisualization::ShowVolumeMounts(const FString& Id){auto* P=ResolveContainer(Id); return IsValid(P)&&P->RenderVolumes();}
bool UTJContainerIsolationVisualization::VisualizeResourceLimits(const FString& Id){auto* P=ResolveContainer(Id); return IsValid(P)&&P->RenderLimits();}
bool UTJContainerIsolationVisualization::ShowContainerHealth(const FString& Id){auto* P=ResolveContainer(Id); return IsValid(P)&&P->RenderHealth();}
