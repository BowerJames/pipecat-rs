# Pipecat RS

Pipecat RS is a Rust project heavily inspired by [Pipecat](https://github.com/pipecat-ai/pipecat/tree/main). It is designed to stand up Agentic server suitable for realtime communication using the following modalities:

1. Text
2. Audio
3. Image

It will support a variety of transport protocols such as:

1. Websockets

## High Level Design

The core components of pipecat rs are:

1. Frame
2. FrameProcessor
3. Pipeline

Frames are instructions and data packets that flow through the system.

Frame processors recieve and process frames. Different types of processors will ingest, process and emit different type of frames. this will govern how the agent behaves. Frame processors could be stateful or stateless and may be predefined or customizable. 

The pipeline is defined by a linear series of frame processors. It will orchestrate the flow of frames through the processors and manage communications with client applications.

### Frame

Frames are split into 2 categories of frames:

1. SystemFrames
2. DataFrames

System Frames are high priority frames that when emitted are broadcasted immediately to all processors in the pipeline and handled with immediate priority.

Data Frames are data packets that flow traditionally through the pipeline. They carry data and instructons linearly through the pipeline until the are processed or reach the start or end of the pipeline. Data Frame can flow both upstream and downstream through the pipeline.

### Frame Processor

Frame processors process frames as they flow through the application and emit new frames to flow through the pipeline. A few examples of frame processors are:

1. InputProcessor - The input processor will recieve incoming frames from the client application and immedately forward them through the pipeline. They can be placed anywhere in the pipeline allowing developers to control where incoming data from a client starts in the pipeline.
2. OutputProcessor - The output processor will process certain frames and send them to the client application. They can be placed anywhere in the pipeline allowing developers to control which points in the pipeline order frames can be emitted back to the client.
3. FilterProcessor - The fitler processor will process certain frames and stop them from flowing further through the application. This allows developers to stop frames flowing too far through an applications.
4. LLMProcessor - The LLM processor will process LLMContext frames. They will contain an LLM service adpater that will run the LLM context frame producing and LLMResponse frame that will be passed downstream through the pipeline.

All frame processor can be initialised to be observable. If a frame processor is observable this means the frames that flow in and out of the processor are tracked and stored but it does not impact the way in which the processor processes or emits frames in any way. This is mostly used for testing pipelines so that the tests can validate the correct frames are processed and emitteed by the processors. 

### Pipeline

Pipelines define the order frames flow through the application. It is defined by a vector of frame processors. Consider a basic pipeline configured for websocket connections. It may consist of the following frame processors in the following order:

1. InputProcessor
2. EchoProcessor
3. OutputProcessor

Here the echo processor is a special processor that converts input frame types to their corresponding output frame types.

In this scenario we may get the following situation:

1. Cient sends input text message through the websocket.
2. This message is converted to an input text frame and passed to the InputProcessor.
3. The InputProcessor will pass this frame downstream through the pipeline.
4. The EchoProcessor will process this frame, absorb it and create an output text frame that is sent downstream through the pipeline.
5. The OutputProcessor will process this frame, absorb it and send the output text message back through the websocket to the client.

# Project Structure

```
/Users/jamesbower/Projects/pipecat-rs/
├── AGENTS.md                   # Project documentation and development guidelines
├── Cargo.lock                  # Dependency lock file
├── Cargo.toml                  # Root workspace configuration
├── pipecat_rs/                 # Main Development Code crate
│   ├── Cargo.toml              # Main crate configuration
│   └── src/                    # Development code (structure flexible)
│       ├── lib.rs             # Main library entry point
│       └── ...                 # Additional modules and development code
├── pipecat_rs_locked/          # Locked functionality crate (Only the Project Owner has write permissions here)
│   ├── Cargo.toml              # Locked crate configuration
│   └── src/
│       ├── lib.rs              # Locked library entry point
│       └── ...                 # Additional locked modules and functionality
├── pipecat_rs_tests/           # Test suite crate
│   ├── Cargo.toml              # Test crate configuration
│   ├── src/
│   │   ├── main.rs             # Test runner entry point
│   │   └── ...                 # Additional test utilities and helpers
│   └── tests/                  # Integration tests
│       └── ...                 # Test files (structure flexible)
└── target/                     # Build artifacts and dependencies
    ├── CACHEDIR.TAG
    └── debug/
        ├── build/              # Build script outputs
        ├── deps/               # Compiled dependencies
        ├── examples/           # Compiled examples
        ├── incremental/        # Incremental compilation cache
        └── libpipecat_rs.*     # Compiled library artifacts
```

This project is split into multiple crates:

1. pipecat_rs_locked - This crate contains code that is only allowed to be edited by the project owner (the user). You are never allowed to edit this unless you are given express permission by the project owner.
2. pipecat_rs - This crate contains the main development code that can be freely edited during the Compiler Driven Development, Dev Code Development, and Refactor stages. This is where the core framework implementation lives.
3. pipecat_rs_tests - This crate contains the comprehensive test suite that serves as specifications for the framework behavior. This crate can be freely edited during the Test Code Development stage. Tests are written at the highest level of abstraction possible and validate the expected behavior of the system.

## Crate Organization

- **`pipecat_rs`**: Main library crate that provides the public API for the pipecat-rs framework
- **`pipecat_rs_core`**: Core functionality crate containing the fundamental types and traits (Frame, FrameProcessor, Pipeline)
- **`pipecat_rs_tests`**: Comprehensive test suite that validates the behavior of the framework at the highest level of abstraction

## Key Files

- **`AGENTS.md`**: This file containing project documentation, architecture overview, and development guidelines
- **`frame.rs`**: Core frame definitions including SystemFrames and DataFrames
- **Test files**: Integration tests that serve as specifications for the framework behavior


# Development Cycle

During development you must follow a very particular development process designed to optimise the effectiveness of ai driven development. The project follows a hybrid compiler driven development, test driven development process that progresses through the following stages:

1. Test Code Development
2. Compiler Driven Development
3. Dev Code Development
4. Refactor

In the first stage tests will be created that test the expected behaviour of the application. These tests act as the specs for the application and should aim to test the behaviour of the application at the highest level of abstraction possible. During this stage no effort is made to make the tests pass or even compile.

In the second stage the development code is edited such that all tests will now compile. No effort is put into making the tests pass, heavy use of stubs and the todo! macro will be used during this stage.

In the third stage the development code will be continually edited until all tests pass. This stage is only complete if all tests pass and you must continue working until you reach this state.

In the final stage you have the ability to refactor the development code. You will perform a review of the current dev code and may refactor the code in any way you wish to improve structure, readability, design and performance. This test code cannot be edited during the refactor and the stage is only considered complete if all the tests pass by the end. If tests do not pass then all edits will be thrown away.

## Test Code Development

When in test code development mode you have the following permissions:

**Crate Permissions:**
- **pipecat_rs_locked**: Read only
- **pipecat_rs**: Read only
- **pipecat_rs_tests**: Read and Write

**Activities:**
1. **Create comprehensive test specifications**: Write tests that serve as behavioral specifications for the framework
2. **Test at highest abstraction level**: Focus on testing the overall system behavior rather than implementation details
3. **No compilation requirements**: Tests do not need to compile or pass during this stage

**Practices:**
- Write tests that describe the expected behavior of the system from a user's perspective
- Use descriptive test names that clearly explain what behavior is being tested
- Group related tests logically and use clear test structure
- Focus on integration tests that test the complete pipeline flow
- Do not worry about making tests compile or pass - this is purely specification writing
- **Use simulated adapter alternatives**: All third-party services (LLMs, APIs, external services) must use simulated/mock adapters to avoid calling real third-party code during test execution

## Compiler Driven Development

When in compiler driven development mode you have the following permissions:

**Crate Permissions:**
- **pipecat_rs_locked**: Read only
- **pipecat_rs**: Read and Write
- **pipecat_rs_tests**: Read only

**Activities:**
1. **Create stubs and placeholders**: Use `todo!()` macros and stub implementations liberally
2. **Define types and interfaces**: Create the necessary type definitions and trait implementations
3. **Make tests compile**: Focus solely on making all tests compile without errors
4. **No functionality requirements**: Code does not need to work correctly, only compile

**Practices:**
- Focus solely on making all tests compile without errors
- Use `todo!()` macros for unimplemented functionality
- Create minimal stub implementations that satisfy type requirements
- Define all necessary types, traits, and function signatures
- Do not implement actual logic - that comes in the next stage
- Ensure all imports and dependencies are correctly specified

## Dev Code Development

When in dev code development mode you have the following permissions:

**Crate Permissions:**
- **pipecat_rs_locked**: Read only
- **pipecat_rs**: Read and Write
- **pipecat_rs_tests**: Read only

**Activities:**
1. **Implement all functionality**: Replace stubs and `todo!()` macros with actual working code
2. **Make all tests pass**: Continuously work until every test passes successfully
3. **Focus on correctness**: Ensure all functionality works as specified by the tests

**Practices:**
- Replace all `todo!()` macros with actual implementations
- Implement the core logic for all frame processors and pipeline components
- Ensure all tests pass before considering this stage complete
- Focus on correctness over optimization (optimization comes in refactor stage)
- Maintain the interface contracts established in the compiler driven development stage
- Test frequently to ensure progress toward passing all tests

## Refactor

When in refactor mode you have the following permissions:

**Crate Permissions:**
- **pipecat_rs_locked**: Read only
- **pipecat_rs**: Read and Write
- **pipecat_rs_tests**: Read only

**Activities:**
1. **Improve code quality**: Focus on structure, readability, design, and performance
2. **Maintain test compatibility**: All tests must continue to pass after refactoring
3. **Optimize performance**: Enhance code efficiency where appropriate
4. **Enhance maintainability**: Improve code organization and modularity

**Practices:**
- Review the current codebase for improvement opportunities
- Refactor for better structure, readability, and maintainability
- Optimize performance where appropriate
- Improve error handling and edge case management
- Enhance code organization and modularity
- Ensure all tests continue to pass after each refactoring change
- If any tests fail after refactoring, revert changes and try a different approach








