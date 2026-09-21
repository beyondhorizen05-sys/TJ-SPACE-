#include "PerformanceStreamingBudget.h"
#include "Math/UnrealMathUtility.h"
#include "Algo/Sort.h"
bool UTJPerformanceStreamingBudget::AllocateFrameBudget(float B){if(B<=0||!FMath::IsFinite(B))return false;TotalBudgetMs=B;Reallocate(B);return true;}
void UTJPerformanceStreamingBudget::RegisterSystem(const FString& S,ETJPerformancePriority P){if(S.IsEmpty())return;FTJFrameBudgetAllocation& A=Allocations.FindOrAdd(S);A.System=S;A.Priority=P;A.BudgetMs=0;Costs.FindOrAdd(S).System=S;Reallocate(TotalBudgetMs);}
void UTJPerformanceStreamingBudget::Reallocate(float B){
 const float weights[]={.35f,.30f,.25f,.10f}; float sum=0; for(const auto& K:Allocations)sum+=weights[(int32)K.Value.Priority]; if(sum<=0)return;
 for(auto& K:Allocations){K.Value.BudgetMs=B*weights[(int32)K.Value.Priority]/sum; K.Value.ReportedMs=Costs.FindRef(K.Key).Ms;}
}
bool UTJPerformanceStreamingBudget::ReportFrameCost(const FString& S,float M){if(S.IsEmpty()||M<0||!FMath::IsFinite(M))return false; if(!Allocations.Contains(S))RegisterSystem(S,ETJPerformancePriority::Normal);Costs[S].Ms=M;Costs[S].bStarved=M>Allocations[S].BudgetMs;Allocations[S].ReportedMs=M;return true;}
TArray<FTJStreamingCandidate> UTJPerformanceStreamingBudget::PrioritizeStreaming(const FTransform& C,float B)const{
 TArray<FTJStreamingCandidate> O; if(B<=0)return O; for(const auto& K:DistrictTransforms){const float D=FVector::Dist(C.GetLocation(),K.Value.GetLocation());FTJStreamingCandidate X;X.DistrictId=K.Key;X.DistanceCm=D;X.ScreenImportance=DistrictImportance.FindRef(K.Key);X.PriorityScore=FMath::Max(0,FMath::RoundToInt(X.ScreenImportance*10000.f-D/100.f));O.Add(X);}
 O.Sort([](const FTJStreamingCandidate&A,const FTJStreamingCandidate&B){return A.PriorityScore>B.PriorityScore;}); return O;
}
void UTJPerformanceStreamingBudget::RegisterStreamingDistrict(const FString& D,const FTransform& T,float I){if(D.IsEmpty()||I<0)return;DistrictTransforms.Add(D,T);DistrictImportance.Add(D,I);}
bool UTJPerformanceStreamingBudget::DemoteLOD(const FString&A,int32 L){if(A.IsEmpty()||L<0)return false;int32& V=LODLevels.FindOrAdd(A);V=FMath::Max(V,L);return true;}
bool UTJPerformanceStreamingBudget::PromoteLOD(const FString&A,int32 L){if(A.IsEmpty()||L<0)return false;int32& V=LODLevels.FindOrAdd(A);V=FMath::Min(V,L);return true;}
bool UTJPerformanceStreamingBudget::EnterPerformanceMode(){if(bPerformanceMode)return true;bPerformanceMode=true;return true;}
bool UTJPerformanceStreamingBudget::ExitPerformanceMode(){bPerformanceMode=false;return true;}
FTJPerformanceReport UTJPerformanceStreamingBudget::GetPerformanceReport()const{FTJPerformanceReport R;R.TargetFrameMs=16.6667f;R.TargetVRFrameMs=11.1111f;R.AllocatedMs=TotalBudgetMs;R.bPerformanceMode=bPerformanceMode;for(const auto&K:Costs)R.Costs.Add(K.Value);for(const auto&K:Allocations)R.Allocations.Add(K.Value);for(const auto&K:R.Costs)R.ReportedMs+=K.Ms;return R;}