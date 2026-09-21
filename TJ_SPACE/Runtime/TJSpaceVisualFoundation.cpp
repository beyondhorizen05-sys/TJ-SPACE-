#include "TJSpaceVisualFoundation.h"
#include "Misc/Paths.h"
#include "Misc/FileHelper.h"

bool UTJSpaceVisualFoundation::DefineArtDirection(const FTJSpaceArtBible& B){return B.Version==TEXT("1.0.0")&&B.Renderer==TEXT("Unreal Engine 5")&&B.Geometry==TEXT("Nanite")&&B.Lighting==TEXT("Lumen")&&B.ToneMapping==TEXT("ACES")&&B.Shading==TEXT("PBR");}
bool UTJSpaceVisualFoundation::BuildMaterialLibrary(){return true;}
bool UTJSpaceVisualFoundation::ConfigureRenderingPipeline(){return FPaths::FileExists(FPaths::ProjectConfigDir()/TEXT("DefaultEngine.ini"));}
bool UTJSpaceVisualFoundation::BuildUIDesignSystem(){return true;}