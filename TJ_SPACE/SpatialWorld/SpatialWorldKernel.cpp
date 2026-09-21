#include "SpatialWorldKernel.h"

#include "Engine/World.h"
#include "WorldPartition/WorldPartition.h"

bool UTJSpatialWorldKernel::InitializeWorldKernel(UWorld* WorldContext)
{
    if (!IsValid(WorldContext))
    {
        return false;
    }

    World = WorldContext;

    if (UWorldPartition* WorldPartition = World->GetWorldPartition())
    {
        if (!WorldPartition->CanStream())
        {
            return false;
        }
    }

    StreamingOrigin = World->GetWorldSettings()
        ? World->GetWorldSettings()->GetActorLocation()
        : FVector::ZeroVector;

    return true;
}

bool UTJSpatialWorldKernel::RegisterSpatialZone(const FTJSpatialZone& Zone)
{
    if (Zone.Id.IsEmpty() ||
        Zone.Bounds.Extent.X <= 0.0 ||
        Zone.Bounds.Extent.Y <= 0.0 ||
        Zone.Bounds.Extent.Z <= 0.0)
    {
        return false;
    }

    for (const FTJSpatialZone& Existing : Zones)
    {
        if (Existing.Id.Equals(Zone.Id, ESearchCase::CaseSensitive))
        {
            return false;
        }
    }

    FTJSpatialZone NewZone = Zone;
    NewZone.bLoaded = false;
    Zones.Add(MoveTemp(NewZone));
    return true;
}

bool UTJSpatialWorldKernel::ResolveZone(const FVector& WorldLocation, FTJSpatialZone& OutZone) const
{
    for (const FTJSpatialZone& Zone : Zones)
    {
        if (Zone.Bounds.Contains(WorldLocation))
        {
            OutZone = Zone;
            return true;
        }
    }

    return false;
}

bool UTJSpatialWorldKernel::SetStreamingOrigin(const FVector& Origin)
{
    if (!Origin.IsFinite())
    {
        return false;
    }

    StreamingOrigin = Origin;
    return true;
}

int32 UTJSpatialWorldKernel::UpdateStreamingState()
{
    int32 ChangedCount = 0;

    for (FTJSpatialZone& Zone : Zones)
    {
        const bool bShouldLoad =
            FVector::DistSquared(StreamingOrigin, Zone.Bounds.Center) <= FMath::Square(LoadingRange);

        if (Zone.bLoaded != bShouldLoad)
        {
            Zone.bLoaded = bShouldLoad;
            OnZoneStateChanged.Broadcast(Zone);
            ++ChangedCount;
        }
    }

    return ChangedCount;
}