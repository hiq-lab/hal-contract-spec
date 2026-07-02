{-# LANGUAGE DeriveGeneric #-}
{-# LANGUAGE OverloadedStrings #-}
{-# LANGUAGE TypeFamilies #-}

-- |
-- Module      : HAL.Contract.Backend
-- Description : HAL Contract v2 (spec v2.3) — Haskell reference implementation
-- License     : Apache-2.0
-- Maintainer  : daniel@arvak.io
--
-- This module provides the type class and data types matching the
-- HAL Contract v2 specification (v2.3). Any quantum backend can
-- implement the 'Backend' type class to participate in orchestrated
-- workflows.
--
-- @
-- Lifecycle:
--   capabilities -> validate -> submit -> status -> result
--    (pure)        (IO)       (IO)      (IO)      (IO)
-- @
module HAL.Contract.Backend
  ( -- * Backend type class
    Backend(..)
    -- * Capabilities
  , Capabilities(..)
  , GateSet(..)
  , Topology(..)
  , TopologyKind(..)
  , NoiseProfile(..)
    -- * Availability & validation
  , BackendAvailability(..)
  , ValidationResult(..)
    -- * Jobs
  , JobId(..)
  , JobStatus(..)
  , isTerminal
  , isPending
  , isSuccess
    -- * Results
  , Counts(..)
  , ExecutionResult(..)
    -- * Errors
  , HalError(..)
  ) where

import Data.Aeson (FromJSON, ToJSON)
import Data.Map.Strict (Map)
import qualified Data.Map.Strict as Map
import Data.Text (Text)
import GHC.Generics (Generic)

-- | Unique identifier for a job.
newtype JobId = JobId { unJobId :: Text }
  deriving (Eq, Ord, Show, Generic)

instance FromJSON JobId
instance ToJSON JobId

-- | Status of a job.
--
-- State machine (§5.1):
--
-- @
--   submit -> Queued -> Running -> Completed -> ResultExpired
--               |          |
--               |          +-> Failed reason
--               |          |
--               +----------+-> Cancelled
-- @
--
-- Transitions are monotonic — a job never moves backward or re-enters
-- execution. 'Failed', 'Cancelled', and 'ResultExpired' are permanent.
-- 'Completed' is terminal for execution but MAY transition to
-- 'ResultExpired' if the backend purges results after a retention
-- window. 'ResultExpired' is only reachable from 'Completed': the job
-- ran successfully but its results are no longer available — it is
-- distinct from 'Failed'.
data JobStatus
  = Queued
  | Running
  | Completed
  | Failed Text
  | Cancelled
  | ResultExpired
  deriving (Eq, Show, Generic)

instance FromJSON JobStatus
instance ToJSON JobStatus

-- | Check if a job status is terminal.
--
-- Note: 'Completed' is terminal for execution, but MAY still
-- transition to 'ResultExpired' (§5.1).
isTerminal :: JobStatus -> Bool
isTerminal Completed     = True
isTerminal (Failed _)    = True
isTerminal Cancelled     = True
isTerminal ResultExpired = True
isTerminal _             = False

-- | Check if a job is still pending.
isPending :: JobStatus -> Bool
isPending Queued  = True
isPending Running = True
isPending _       = False

-- | Check if a job completed successfully with results available.
--
-- 'ResultExpired' jobs also ran successfully, but their results have
-- been purged, so 'jobResult' is no longer valid for them.
isSuccess :: JobStatus -> Bool
isSuccess Completed = True
isSuccess _         = False

-- | Kind of qubit topology.
data TopologyKind
  = FullyConnected
  | Linear
  | Star
  | Grid { gridRows :: Int, gridCols :: Int }
  | HeavyHex
  | Custom
  | NeutralAtom { zones :: Int }
  deriving (Eq, Show, Generic)

instance FromJSON TopologyKind
instance ToJSON TopologyKind

-- | Qubit connectivity topology. All edges are bidirectional.
data Topology = Topology
  { topologyKind  :: TopologyKind
  , topologyEdges :: [(Int, Int)]
  } deriving (Eq, Show, Generic)

instance FromJSON Topology
instance ToJSON Topology

-- | Gate set supported by a backend.
--
-- Gate names follow OpenQASM 3 convention (lowercase).
-- If 'gateSetNative' is empty, all supported gates are native.
data GateSet = GateSet
  { gateSetSingleQubit :: [Text]
  , gateSetTwoQubit    :: [Text]
  , gateSetThreeQubit  :: [Text]
  , gateSetNative      :: [Text]
  } deriving (Eq, Show, Generic)

instance FromJSON GateSet
instance ToJSON GateSet

-- | Device-wide noise averages.
--
-- Fidelity values in [0.0, 1.0] (1.0 = perfect).
-- Time values in microseconds.
data NoiseProfile = NoiseProfile
  { npT1                 :: Maybe Double
  , npT2                 :: Maybe Double
  , npSingleQubitFidelity :: Maybe Double
  , npTwoQubitFidelity   :: Maybe Double
  , npReadoutFidelity    :: Maybe Double
  , npGateTime           :: Maybe Double
  } deriving (Eq, Show, Generic)

instance FromJSON NoiseProfile
instance ToJSON NoiseProfile

-- | Hardware capabilities of a quantum backend.
data Capabilities = Capabilities
  { capName         :: Text
  , capNumQubits    :: Int
  , capGateSet      :: GateSet
  , capTopology     :: Topology
  , capMaxShots     :: Int
  , capMaxCircuitOps :: Maybe Int
    -- ^ Maximum gate operations per circuit ('Nothing' = no limit).
  , capIsSimulator  :: Bool
  , capFeatures     :: [Text]
  , capNoiseProfile :: Maybe NoiseProfile
  } deriving (Eq, Show, Generic)

instance FromJSON Capabilities
instance ToJSON Capabilities

-- | Backend availability information.
data BackendAvailability = BackendAvailability
  { availIsAvailable     :: Bool
  , availQueueDepth      :: Maybe Int
  , availEstimatedWait   :: Maybe Double  -- ^ Seconds
  , availStatusMessage   :: Maybe Text
  } deriving (Eq, Show, Generic)

instance FromJSON BackendAvailability
instance ToJSON BackendAvailability

-- | Result of circuit validation.
data ValidationResult
  = Valid
  | Invalid [Text]
  | RequiresTranspilation Text
  deriving (Eq, Show, Generic)

instance FromJSON ValidationResult
instance ToJSON ValidationResult

-- | Measurement counts from circuit execution.
--
-- Bitstring ordering follows OpenQASM 3 convention:
-- rightmost bit = lowest qubit index.
newtype Counts = Counts { unCounts :: Map Text Int }
  deriving (Eq, Show, Generic)

instance FromJSON Counts
instance ToJSON Counts

-- | Result of circuit execution.
data ExecutionResult = ExecutionResult
  { erCounts          :: Counts
  , erShots           :: Int
  , erExecutionTimeMs :: Maybe Int
  , erMetadata        :: Maybe Text  -- ^ JSON metadata as text
  } deriving (Eq, Show, Generic)

instance FromJSON ExecutionResult
instance ToJSON ExecutionResult

-- | Errors that can occur in HAL operations.
--
-- Error taxonomy (§7), 14 variants:
--
-- * Transient (retry with backoff): 'ErrBackendUnavailable', 'ErrTimeout'
-- * Permanent (fix input): 'ErrInvalidCircuit', 'ErrCircuitTooLarge',
--   'ErrInvalidShots', 'ErrUnsupported'
-- * Job-level (resubmit or abort): 'ErrSubmissionFailed', 'ErrJobFailed',
--   'ErrJobCancelled', 'ErrJobNotFound', 'ErrResultExpired'
-- * Auth (re-authenticate): 'ErrAuthenticationFailed'
-- * Config (fix configuration): 'ErrConfiguration', 'ErrBackend'
data HalError
  = ErrBackendUnavailable Text
  | ErrTimeout Text
  | ErrInvalidCircuit Text
  | ErrCircuitTooLarge Text
  | ErrInvalidShots Text
  | ErrUnsupported Text
  | ErrSubmissionFailed Text
  | ErrJobFailed Text
  | ErrJobCancelled
  | ErrJobNotFound Text
  | ErrResultExpired Text
    -- ^ The job completed but its results are no longer available
    -- (§5.1 'ResultExpired').
  | ErrAuthenticationFailed Text
  | ErrConfiguration Text
  | ErrBackend Text
  deriving (Eq, Show, Generic)

-- | Type class for quantum backends.
--
-- Implements the HAL Contract v2 specification. The circuit type @c@
-- is a type parameter, making the interface independent of any
-- specific intermediate representation.
--
-- @
-- Lifecycle:
--   capabilities -> validate -> submit -> status -> result
--    (pure)        (IO)       (IO)      (IO)      (IO)
-- @
class Backend b where
  -- | The circuit type this backend accepts.
  type Circuit b

  -- | Get the name of this backend.
  backendName :: b -> Text

  -- | Get the capabilities of this backend.
  -- MUST be pure and return cached capabilities.
  capabilities :: b -> Capabilities

  -- | Check backend availability.
  availability :: b -> IO (Either HalError BackendAvailability)

  -- | Validate a circuit and shot count against backend constraints
  -- (§3.3 rule 3). Implementations MUST check at minimum:
  --
  -- * qubit count vs 'capNumQubits'
  -- * shot count vs 'capMaxShots'
  -- * gate support vs 'capGateSet'
  -- * if 'capMaxCircuitOps' is set, total operation count vs that limit
  validate :: b -> Circuit b -> Int -> IO (Either HalError ValidationResult)

  -- | Submit a circuit for execution with the given shot count.
  -- MUST call 'validate' internally before dispatching; an 'Invalid'
  -- circuit MUST yield 'ErrInvalidCircuit' without submitting.
  submit :: b -> Circuit b -> Int -> IO (Either HalError JobId)

  -- | Submit a parametric circuit, binding named parameter values
  -- (OpenQASM 3.0 @input float[64]@ declarations) before dispatch
  -- (§3.3 rule 8).
  --
  -- The default implementation delegates to 'submit' when the
  -- parameter map is empty and returns 'ErrUnsupported' otherwise.
  -- Backends that support parametric execution SHOULD override it;
  -- overrides are subject to the same validation requirements as
  -- 'submit'.
  submitWithParameters
    :: b -> Circuit b -> Int -> Map Text Double
    -> IO (Either HalError JobId)
  submitWithParameters backend circuit shots parameters
    | Map.null parameters = submit backend circuit shots
    | otherwise =
        pure (Left (ErrUnsupported "parametric circuits not supported by this backend"))

  -- | Get the status of a job.
  jobStatus :: b -> JobId -> IO (Either HalError JobStatus)

  -- | Get the result of a completed job.
  --
  -- Only valid when 'jobStatus' returns 'Completed'. If the job is
  -- 'ResultExpired', implementations MUST return 'ErrResultExpired'
  -- (§3.3 rule 5).
  jobResult :: b -> JobId -> IO (Either HalError ExecutionResult)

  -- | Cancel a running job. Best-effort — the job may have already
  -- reached a terminal state.
  cancel :: b -> JobId -> IO (Either HalError ())
