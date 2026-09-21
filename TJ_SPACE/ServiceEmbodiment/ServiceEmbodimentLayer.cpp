#include "ServiceEmbodimentLayer.h"

#include "Engine/World.h"
#include "Materials/MaterialInstanceDynamic.h"
#include "UObject/ConstructorHelpers.h"

ATJServiceBuilding::ATJServiceBuilding()
{
    PrimaryActorTick.bCanEverTick = true;

    Root = CreateDefaultSubobject<USceneComponent>(TEXT("Root"));
    SetRootComponent(Root);

    Exterior = CreateDefaultSubobject<UStaticMeshComponent>(TEXT("Exterior"));
    Exterior->SetupAttachment(Root);
    Exterior->SetCollisionEnabled(ECollisionEnabled::QueryAndPhysics);

    InteriorRoot = CreateDefaultSubobject<USceneComponent>(TEXT("InteriorRoot"));
    InteriorRoot->SetupAttachment(Root);
}

void ATJServiceBuilding::ConfigureManifest(const FTJServiceManifest& InManifest)
{
    Manifest = InManifest;
    BaseScale = Manifest.BuildingScale;
    SetActorScale3D(BaseScale);
}

bool ATJServiceBuilding::BuildShell()
{
    if (!Manifest.BuildingMesh.IsValid())
    {
        return false;
    }

    UStaticMesh* Mesh = Cast<UStaticMesh>(Manifest.BuildingMesh.TryLoad());
    if (!IsValid(Mesh))
    {
        return false;
    }

    Exterior->SetStaticMesh(Mesh);
    Exterior->SetRelativeScale3D(FVector::OneVector);
    return true;
}

bool ATJServiceBuilding::BuildInterior()
{
    if (!Manifest.InteriorMesh.IsValid() && Manifest.InteriorModuleMeshes.IsEmpty())
    {
        return false;
    }

    if (Manifest.InteriorMesh.IsValid())
    {
        if (UStaticMesh* Mesh = Cast<UStaticMesh>(Manifest.InteriorMesh.TryLoad()))
        {
            UStaticMeshComponent* Interior = NewObject<UStaticMeshComponent>(this);
            Interior->RegisterComponent();
            Interior->AttachToComponent(InteriorRoot, FAttachmentTransformRules::KeepRelativeTransform);
            Interior->SetStaticMesh(Mesh);
            Interior->SetRelativeScale3D(Manifest.InteriorScale);
            Interior->SetCollisionEnabled(ECollisionEnabled::QueryAndPhysics);
            InteriorModules.Add(Interior);
        }
    }

    for (const FSoftObjectPath& ModulePath : Manifest.InteriorModuleMeshes)
    {
        if (UStaticMesh* Mesh = Cast<UStaticMesh>(ModulePath.TryLoad()))
        {
            UStaticMeshComponent* Module = NewObject<UStaticMeshComponent>(this);
            Module->RegisterComponent();
            Module->AttachToComponent(InteriorRoot, FAttachmentTransformRules::KeepRelativeTransform);
            Module->SetStaticMesh(Mesh);
            Module->SetCollisionEnabled(ECollisionEnabled::QueryAndPhysics);
            InteriorModules.Add(Module);
        }
    }

    return InteriorModules.Num() > 0;
}

bool ATJServiceBuilding::BuildInterfaces()
{
    InterfaceExits.Reset();

    for (int32 Index = 0; Index < Manifest.Interfaces.Num(); ++Index)
    {
        const FTJServiceInterfaceManifest& Interface = Manifest.Interfaces[Index];

        UStaticMeshComponent* Exit = NewObject<UStaticMeshComponent>(this);
        Exit->RegisterComponent();
        Exit->AttachToComponent(Root, FAttachmentTransformRules::KeepRelativeTransform);

        const float Angle = Manifest.Interfaces.Num() > 0
            ? (360.0f * static_cast<float>(Index) / Manifest.Interfaces.Num())
            : 0.0f;

        const float Radians = FMath::DegreesToRadians(Angle);
        const FVector Location(FMath::Cos(Radians) * 450.0f, FMath::Sin(Radians) * 450.0f, 120.0f);
        Exit->SetRelativeLocation(Location);
        Exit->SetRelativeRotation(FRotator(0.0f, Angle, 0.0f));
        Exit->SetRelativeScale3D(FVector(0.35f, 0.35f, 1.5f));
        Exit->SetCollisionEnabled(ECollisionEnabled::NoCollision);
        InterfaceExits.Add(Exit);

        SetMaterialScalar(Exit, TEXT("InterfaceActive"), 1.0f);
        SetMaterialScalar(Exit, TEXT("InterfacePort"), FMath::Clamp(Interface.Port / 65535.0f, 0.0f, 1.0f));
    }

    return true;
}

bool ATJServiceBuilding::BuildDependencies()
{
    DependencyPipes.Reset();

    for (int32 Index = 0; Index < Manifest.Dependencies.Num(); ++Index)
    {
        const FTJServiceDependencyManifest& Dependency = Manifest.Dependencies[Index];

        UStaticMeshComponent* Pipe = NewObject<UStaticMeshComponent>(this);
        Pipe->RegisterComponent();
        Pipe->AttachToComponent(Root, FAttachmentTransformRules::KeepRelativeTransform);

        const float Offset = (static_cast<float>(Index) - Manifest.Dependencies.Num() * 0.5f) * 55.0f;
        Pipe->SetRelativeLocation(FVector(0.0f, Offset, -90.0f));
        Pipe->SetRelativeScale3D(FVector(1.4f, 0.08f, 0.08f));
        Pipe->SetCollisionEnabled(ECollisionEnabled::NoCollision);
        DependencyPipes.Add(Pipe);

        SetMaterialScalar(Pipe, TEXT("DependencyFlow01"), FMath::Clamp(static_cast<float>(Dependency.Throughput01), 0.0f, 1.0f));
    }

    return true;
}

bool ATJServiceBuilding::BuildResourceFootprint()
{
    ResourceFootprint.Reset();

    const float Radius = 300.0f +
        static_cast<float>(Manifest.Resources.CpuCores * 20.0 +
                           Manifest.Resources.MemoryGB * 3.0 +
                           Manifest.Resources.StorageGB * 0.15);

    UStaticMeshComponent* Footprint = NewObject<UStaticMeshComponent>(this);
    Footprint->RegisterComponent();
    Footprint->AttachToComponent(Root, FAttachmentTransformRules::KeepRelativeTransform);
    Footprint->SetRelativeLocation(FVector(0.0f, 0.0f, -115.0f));
    Footprint->SetRelativeScale3D(FVector(Radius / 100.0f));
    Footprint->SetCollisionEnabled(ECollisionEnabled::NoCollision);
    ResourceFootprint.Add(Footprint);

    SetMaterialScalar(Footprint, TEXT("ResourceIntensity01"),
        FMath::Clamp(static_cast<float>(
            (Manifest.Resources.CpuCores / 64.0) +
            (Manifest.Resources.MemoryGB / 256.0) +
            (Manifest.Resources.StorageGB / 4096.0) +
            (Manifest.Resources.NetworkMbps / 10000.0)) / 4.0f, 0.0f, 1.0f));

    return true;
}

void ATJServiceBuilding::BeginStateTransition(ETJServiceState NewState)
{
    if (bArchived)
    {
        return;
    }

    if (CurrentState == NewState && !bTransitioning)
    {
        return;
    }

    TargetState = NewState;
    ActiveTransition = MakeTransition(CurrentState, TargetState);
    TransitionElapsed = 0.0f;
    bTransitioning = ActiveTransition.DurationSeconds > 0.0f;

    if (!bTransitioning)
    {
        CurrentState = TargetState;
        ApplyArchitecturalState(1.0f);
    }
}

void ATJServiceBuilding::Tick(float DeltaSeconds)
{
    Super::Tick(DeltaSeconds);

    if (!bTransitioning)
    {
        return;
    }

    TransitionElapsed = FMath::Min(
        TransitionElapsed + FMath::Max(DeltaSeconds, 0.0f),
        ActiveTransition.DurationSeconds);

    const float Alpha = EvaluateCurve(
        ActiveTransition.Curve,
        ActiveTransition.DurationSeconds > 0.0f
            ? TransitionElapsed / ActiveTransition.DurationSeconds
            : 1.0f);

    ApplyArchitecturalState(Alpha);

    if (TransitionElapsed >= ActiveTransition.DurationSeconds)
    {
        CurrentState = TargetState;
        bTransitioning = false;
        ApplyArchitecturalState(1.0f);
    }
}

void ATJServiceBuilding::ApplyArchitecturalState(float Alpha)
{
    const float A = FMath::Clamp(Alpha, 0.0f, 1.0f);
    const float FromStress =
        CurrentState == ETJServiceState::Error ? 1.0f :
        CurrentState == ETJServiceState::Stopped ? 0.85f :
        CurrentState == ETJServiceState::Updating ? 0.45f :
        CurrentState == ETJServiceState::Starting ? 0.20f : 0.0f;

    const float ToStress =
        TargetState == ETJServiceState::Error ? 1.0f :
        TargetState == ETJServiceState::Stopped ? 0.85f :
        TargetState == ETJServiceState::Updating ? 0.45f :
        TargetState == ETJServiceState::Starting ? 0.20f : 0.0f;

    const float Stress = FMath::Lerp(FromStress, ToStress, A);
    const float Activity = 1.0f - Stress;

    SetMaterialScalar(Exterior, TEXT("ServiceStateBlend"), A);
    SetMaterialScalar(Exterior, TEXT("ServiceStress01"), Stress);
    SetMaterialScalar(Exterior, TEXT("ServiceActivity01"), Activity);

    for (UStaticMeshComponent* Component : InterfaceExits)
    {
        SetMaterialScalar(Component, TEXT("ServiceStateBlend"), A);
        SetMaterialScalar(Component, TEXT("ServiceActivity01"), Activity);
    }

    for (UStaticMeshComponent* Component : DependencyPipes)
    {
        SetMaterialScalar(Component, TEXT("DependencyActivity01"), Activity);
    }

    for (UStaticMeshComponent* Component : ResourceFootprint)
    {
        SetMaterialScalar(Component, TEXT("ResourceActivity01"), Activity);
    }

    const float VerticalPulse =
        TargetState == ETJServiceState::Starting ? 0.035f :
        TargetState == ETJServiceState::Updating ? 0.020f :
        TargetState == ETJServiceState::Error ? -0.015f : 0.0f;

    SetActorScale3D(BaseScale * (1.0f + VerticalPulse * FMath::Sin(A * PI)));
}

void ATJServiceBuilding::Archive()
{
    bArchived = true;
    bTransitioning = false;

    SetActorHiddenInGame(true);
    SetActorEnableCollision(false);
    SetActorTickEnabled(false);

    for (UStaticMeshComponent* Component : InteriorModules)
    {
        if (IsValid(Component))
        {
            Component->SetVisibility(false, true);
            Component->SetCollisionEnabled(ECollisionEnabled::NoCollision);
        }
    }
}

float ATJServiceBuilding::EvaluateCurve(ETJServiceTransitionCurve Curve, float Alpha)
{
    const float X = FMath::Clamp(Alpha, 0.0f, 1.0f);

    switch (Curve)
    {
        case ETJServiceTransitionCurve::Linear:
            return X;

        case ETJServiceTransitionCurve::QuinticEaseOut:
        {
            const float Inv = 1.0f - X;
            return 1.0f - Inv * Inv * Inv * Inv * Inv;
        }

        case ETJServiceTransitionCurve::CubicEaseOut:
        {
            const float Inv = 1.0f - X;
            return 1.0f - Inv * Inv * Inv;
        }

        case ETJServiceTransitionCurve::CubicEaseInOut:
        default:
            return X < 0.5f
                ? 4.0f * X * X * X
                : 1.0f - FMath::Pow(-2.0f * X + 2.0f, 3.0f) / 2.0f;
    }
}

FTJServiceTransitionSpec ATJServiceBuilding::MakeTransition(
    ETJServiceState From,
    ETJServiceState To)
{
    FTJServiceTransitionSpec Result;
    Result.From = From;
    Result.To = To;

    if (From == To)
    {
        Result.Curve = ETJServiceTransitionCurve::Linear;
        Result.DurationSeconds = 0.0f;
        return Result;
    }

    const uint8 F = static_cast<uint8>(From);
    const uint8 T = static_cast<uint8>(To);
    const uint8 Key = static_cast<uint8>(F * 5 + T);

    switch (Key)
    {
        case 1:  Result.Curve = ETJServiceTransitionCurve::CubicEaseInOut; Result.DurationSeconds = 1.20f; break; // stopped -> starting
        case 2:  Result.Curve = ETJServiceTransitionCurve::QuinticEaseOut; Result.DurationSeconds = 2.40f; break; // stopped -> running
        case 3:  Result.Curve = ETJServiceTransitionCurve::CubicEaseInOut; Result.DurationSeconds = 0.90f; break; // stopped -> error
        case 4:  Result.Curve = ETJServiceTransitionCurve::CubicEaseInOut; Result.DurationSeconds = 1.40f; break; // stopped -> updating

        case 5:  Result.Curve = ETJServiceTransitionCurve::CubicEaseInOut; Result.DurationSeconds = 0.80f; break; // starting -> stopped
        case 7:  Result.Curve = ETJServiceTransitionCurve::CubicEaseOut; Result.DurationSeconds = 1.10f; break; // starting -> error
        case 8:  Result.Curve = ETJServiceTransitionCurve::CubicEaseInOut; Result.DurationSeconds = 0.95f; break; // starting -> updating

        case 10: Result.Curve = ETJServiceTransitionCurve::QuinticEaseOut; Result.DurationSeconds = 0.75f; break; // running -> stopped
        case 11: Result.Curve = ETJServiceTransitionCurve::CubicEaseInOut; Result.DurationSeconds = 0.70f; break; // running -> starting
        case 13: Result.Curve = ETJServiceTransitionCurve::CubicEaseInOut; Result.DurationSeconds = 0.65f; break; // running -> error
        case 14: Result.Curve = ETJServiceTransitionCurve::CubicEaseInOut; Result.DurationSeconds = 1.00f; break; // running -> updating

        case 15: Result.Curve = ETJServiceTransitionCurve::CubicEaseInOut; Result.DurationSeconds = 1.30f; break; // error -> stopped
        case 16: Result.Curve = ETJServiceTransitionCurve::CubicEaseOut; Result.DurationSeconds = 1.00f; break; // error -> starting
        case 17: Result.Curve = ETJServiceTransitionCurve::QuinticEaseOut; Result.DurationSeconds = 2.00f; break; // error -> running
        case 19: Result.Curve = ETJServiceTransitionCurve::CubicEaseInOut; Result.DurationSeconds = 0.90f; break; // error -> updating

        case 20: Result.Curve = ETJServiceTransitionCurve::CubicEaseInOut; Result.DurationSeconds = 1.00f; break; // updating -> stopped
        case 21: Result.Curve = ETJServiceTransitionCurve::CubicEaseInOut; Result.DurationSeconds = 0.85f; break; // updating -> starting
        case 22: Result.Curve = ETJServiceTransitionCurve::QuinticEaseOut; Result.DurationSeconds = 1.60f; break; // updating -> running
        case 23: Result.Curve = ETJServiceTransitionCurve::CubicEaseOut; Result.DurationSeconds = 0.80f; break; // updating -> error

        default:
            Result.Curve = ETJServiceTransitionCurve::CubicEaseInOut;
            Result.DurationSeconds = 1.0f;
            break;
    }

    return Result;
}

void ATJServiceBuilding::SetMaterialScalar(
    UPrimitiveComponent* Component,
    FName Parameter,
    float Value) const
{
    if (!IsValid(Component))
    {
        return;
    }

    const int32 Count = Component->GetNumMaterials();
    for (int32 Index = 0; Index < Count; ++Index)
    {
        if (UMaterialInstanceDynamic* MID = Component->CreateAndSetMaterialInstanceDynamic(Index))
        {
            MID->SetScalarParameterValue(Parameter, Value);
        }
    }
}

ATJServiceBuilding* UTJServiceEmbodimentLayer::InstantiateServiceBuilding(
    const FString& PackageId,
    const FTJServiceManifest& Manifest)
{
    if (PackageId.IsEmpty() || Manifest.PackageId != PackageId)
    {
        return nullptr;
    }

    UWorld* World = GetWorld();
    if (!IsValid(World))
    {
        return nullptr;
    }

    if (Buildings.Contains(PackageId))
    {
        return Buildings.FindRef(PackageId);
    }

    ATJServiceBuilding* Building = World->SpawnActor<ATJServiceBuilding>(
        ATJServiceBuilding::StaticClass(),
        FTransform::Identity);

    if (!IsValid(Building))
    {
        return nullptr;
    }

    Building->ConfigureManifest(Manifest);

    if (!Building->BuildShell())
    {
        Building->Destroy();
        return nullptr;
    }

    Building->BuildInterior();
    Building->BuildInterfaces();
    Building->BuildDependencies();
    Building->BuildResourceFootprint();

    Buildings.Add(PackageId, Building);

    return Building;
}

bool UTJServiceEmbodimentLayer::BindServiceState(
    const FString& PackageId,
    ETJServiceState State)
{
    ATJServiceBuilding* Building = ResolveServiceBuilding(PackageId);
    if (!IsValid(Building) || Building->IsArchived())
    {
        return false;
    }

    const ETJServiceState Previous = Building->GetServiceState();
    Building->BeginStateTransition(State);

    OnServiceStateTransitioned.Broadcast(Previous, State);
    return true;
}

bool UTJServiceEmbodimentLayer::RenderServiceInterior(const FString& PackageId)
{
    ATJServiceBuilding* Building = ResolveServiceBuilding(PackageId);
    return IsValid(Building) && Building->BuildInterior();
}

bool UTJServiceEmbodimentLayer::DisplayServiceInterfaces(const FString& PackageId)
{
    ATJServiceBuilding* Building = ResolveServiceBuilding(PackageId);
    return IsValid(Building) && Building->BuildInterfaces();
}

bool UTJServiceEmbodimentLayer::ShowServiceDependencies(const FString& PackageId)
{
    ATJServiceBuilding* Building = ResolveServiceBuilding(PackageId);
    return IsValid(Building) && Building->BuildDependencies();
}

bool UTJServiceEmbodimentLayer::ShowResourceDraw(const FString& PackageId)
{
    ATJServiceBuilding* Building = ResolveServiceBuilding(PackageId);
    return IsValid(Building) && Building->BuildResourceFootprint();
}

bool UTJServiceEmbodimentLayer::ArchiveServiceBuilding(const FString& PackageId)
{
    ATJServiceBuilding* Building = ResolveServiceBuilding(PackageId);
    if (!IsValid(Building))
    {
        return false;
    }

    Building->Archive();
    OnServiceBuildingArchived.Broadcast(PackageId, true);
    return true;
}

ATJServiceBuilding* UTJServiceEmbodimentLayer::ResolveServiceBuilding(
    const FString& PackageId) const
{
    return Buildings.FindRef(PackageId);
}
