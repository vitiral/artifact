module LintConfig exposing (config)

import Lint.Types exposing (LintRule, Severity(..))
import Lint.Rules.DefaultPatternPosition
import Lint.Rules.NoConstantCondition
import Lint.Rules.NoDebug
import Lint.Rules.NoDuplicateImports
import Lint.Rules.NoExposingEverything
import Lint.Rules.NoImportingEverything
import Lint.Rules.NoNestedLet
import Lint.Rules.NoUnannotatedFunction
import Lint.Rules.NoUnusedVariables
import Lint.Rules.NoUselessIf
import Lint.Rules.NoUselessPatternMatching
import Lint.Rules.NoWarningComments
import Lint.Rules.SimplifyPiping
import Lint.Rules.SimplifyPropertyAccess
import Lint.Rules.ElmTest.NoDuplicateTestBodies


config : List ( Severity, LintRule )
config =
    [ ( Critical, Lint.Rules.DefaultPatternPosition.rule { position = Lint.Rules.DefaultPatternPosition.Last } )
    , ( Critical, Lint.Rules.NoConstantCondition.rule )
    , ( Critical, Lint.Rules.NoDebug.rule )
    , ( Critical, Lint.Rules.NoDuplicateImports.rule )
    , ( Critical, Lint.Rules.NoExposingEverything.rule )
    , ( Critical, Lint.Rules.NoImportingEverything.rule { exceptions = [ "Html" ] } )
    , ( Critical, Lint.Rules.NoNestedLet.rule )
    , ( Critical, Lint.Rules.NoUnannotatedFunction.rule )
    , ( Critical, Lint.Rules.NoUnusedVariables.rule )
    , ( Critical, Lint.Rules.NoUselessIf.rule )
    , ( Critical, Lint.Rules.NoUselessPatternMatching.rule )
    , ( Warning, Lint.Rules.NoWarningComments.rule )
    , ( Critical, Lint.Rules.SimplifyPiping.rule )
    , ( Critical, Lint.Rules.SimplifyPropertyAccess.rule )
    , ( Critical, Lint.Rules.ElmTest.NoDuplicateTestBodies.rule )
    ]
