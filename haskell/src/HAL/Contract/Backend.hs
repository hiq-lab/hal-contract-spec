{-# LANGUAGE DeriveGeneric #-}
{-# LANGUAGE OverloadedStrings #-}

-- |
-- Module      : HAL.Contract.Backend
-- Description : HAL Contract v2 — Haskell reference implementation
-- License     : Apache-2.0
-- Maintainer  : daniel@arvak.io
--
-- This module provides the type class and data types matching the
-- HAL Contract v2 specification. Any quantum backend can implement
-- the 'Backend' type class to participate in orchestrated workflows.
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
import Data.Text (Text)
import GHC.Generics (Generic)

-- | Unique identifier for a job.
newtype JobId = JobId { unJobId :: Text }
  deriving (Eq, Ord, Show, Generic)

instance FromJSON JobId
instance ToJSON JobId

-- | Status of a job.
--
-- State machine:
--
-- @
--   submit -> Queued -> Running -> Completed
--               |          |
--               |          +-> Failed reason
--               |          |
--               +----------+-> Cancelled
-- @
data JobStatus
  = Queued
  | Running
  | Completed
  | Failed Text
  | Cancelled
  deriving (Eq, Show, Generic)

instance FromJSON JobStatus
instance ToJSON JobStatus

-- | Check if a job status is terminal.
isTerminal :: JobStatus -> Bool
isTerminal Completed   = True
isTerminal (Failed _)  = True
isTerminal Cancelled   = True
isTerminal _           = False

-- | Check if a job is still pending.
isPending :: JobStatus -> Bool
isPending Queued  = True
isPending Running = True
isPending _       = False

-- | Check if a job completed successfully.
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

  -- | Validate a circuit against backend constraints.
  validate :: b -> Circuit b -> IO (Either HalError ValidationResult)

  -- | Submit a circuit for execution.
  submit :: b -> Circuit b -> Int -> IO (Either HalError JobId)

  -- | Get the status of a job.
  jobStatus :: b -> JobId -> IO (Either HalError JobStatus)

  -- | Get the result of a completed job.
  jobResult :: b -> JobId -> IO (Either HalError ExecutionResult)

  -- | Cancel a running job.
  cancel :: b -> JobId -> IO (Either HalError ())
