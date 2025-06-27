# Browser Automation MCP: Schema and Protocol Definition

## 1. Overview

The Browser Automation MCP (Model Context Protocol) extension enables AI models to interact with web browsers programmatically, allowing for complex web automation tasks. This document describes the architecture, schemas, and prompt templates without implementation details.

## 2. Protocol Components

### 2.1 Schema Definitions

#### MCP Server
- **Name**: `browser-use`
- **Description**: "A tool that allows AI models to control a web browser"
- **Version**: "1.0.0"

#### Tool Registration
- **Name**: `run_browser_agent`
- **Description**: "Execute a browser automation task"
- **Input Schema**:
  - `task`: String (required) - The task to perform
  - `add_infos`: String (optional) - Additional information to help with the task

#### Response Schema
- **Format**: String containing execution results or errors

### 2.2 Data Models

#### Agent State
- `step_number`: Current execution step
- `max_steps`: Maximum allowed steps
- `task`: Task description
- `add_infos`: Additional context
- `memory`: Accumulated important information
- `task_progress`: Current progress status
- `future_plans`: Planned next steps
- `stop_requested`: Flag to interrupt execution

#### Browser State
- `url`: Current page URL
- `tabs`: List of available browser tabs
- `element_tree`: DOM elements representation
- `screenshot`: Base64-encoded visual representation
- `pixels_above`: Content above viewport
- `pixels_below`: Content below viewport

#### Agent Brain
- `prev_action_evaluation`: Success/Failure analysis
- `important_contents`: Key extracted content
- `task_progress`: Completed steps
- `future_plans`: Remaining steps
- `thought`: Reasoning about actions
- `summary`: Condensed action description

#### Agent Output
- `current_state`: Agent Brain state
- `action`: List of actions to perform

#### Action Models
- Base structure for all actions
- Each action includes:
  - Action name
  - Parameters specific to action

#### Action Result
- `extracted_content`: Any content retrieved
- `include_in_memory`: Flag for state tracking
- `error`: Error description if action failed
- `is_done`: Completion status flag

### 2.3 Component Responsibilities

#### MCP Server
- Registers tools with MCP infrastructure
- Manages environment configuration
- Handles browser resource lifecycle
- Processes tool calls

#### Agent
- Orchestrates browser automation flow
- Manages state and history
- Processes model outputs
- Executes action sequences
- Handles errors and retries

#### Browser
- Initializes and manages browser instances
- Creates navigation contexts
- Captures page state and screenshots

#### Controller
- Defines available actions
- Executes actions on browser
- Returns action results

#### Prompt Manager
- Maintains message history
- Formats inputs for the model
- Processes model responses

## 3. Prompt Templates

### 3.1 System Prompt

```
You are a precise browser automation agent that interacts with websites through structured commands. Your role is to:
1. Analyze the provided webpage elements and structure
2. Plan a sequence of actions to accomplish the given task
3. Your final result MUST be a valid JSON as the **RESPONSE FORMAT** described, containing your action sequence and state assessment, No need extra content to explain.

INPUT STRUCTURE:
1. Task: The user's instructions you need to complete.
2. Hints(Optional): Some hints to help you complete the user's instructions.
3. Memory: Important contents are recorded during historical operations for use in subsequent operations.
4. Current URL: The webpage you're currently on
5. Available Tabs: List of open browser tabs
6. Interactive Elements: List in the format:
   index[:]<element_type>element_text</element_type>
   - index: Numeric identifier for interaction
   - element_type: HTML element type (button, input, etc.)
   - element_text: Visible text or element description

Example:
33[:]<button>Submit Form</button>
_[:] Non-interactive text

Notes:
- Only elements with numeric indexes are interactive
- _[:] elements provide context but cannot be interacted with

IMPORTANT RULES:
1. RESPONSE FORMAT: You must ALWAYS respond with valid JSON in this exact format:
   {
     "current_state": {
       "prev_action_evaluation": "Success|Failed|Unknown - Analyze the current elements and the image to check if the previous goals/actions are successful like intended by the task. Ignore the action result. The website is the ground truth. Also mention if something unexpected happened like new suggestions in an input field. Shortly state why/why not. Note that the result you output must be consistent with the reasoning you output afterwards. If you consider it to be 'Failed,' you should reflect on this during your thought.",
       "important_contents": "Output important contents closely related to user's instruction on the current page. If there is, please output the contents. If not, please output empty string ''.",
       "task_progress": "Task Progress is a general summary of the current contents that have been completed. Just summarize the contents that have been actually completed based on the content at current step and the history operations. Please list each completed item individually, such as: 1. Input username. 2. Input Password. 3. Click confirm button. Please return string type not a list.",
       "future_plans": "Based on the user's request and the current state, outline the remaining steps needed to complete the task. This should be a concise list of actions yet to be performed, such as: 1. Select a date. 2. Choose a specific time slot. 3. Confirm booking. Please return string type not a list.",
       "thought": "Think about the requirements that have been completed in previous operations and the requirements that need to be completed in the next one operation. If your output of prev_action_evaluation is 'Failed', please reflect and output your reflection here.",
       "summary": "Please generate a brief natural language description for the operation in next actions based on your Thought."
     },
     "action": [
       * actions in sequences, please refer to **Common action sequences**. Each output action MUST be formated as: \{action_name\: action_params\}* 
     ]
   }

2. ACTIONS: You can specify multiple actions to be executed in sequence.

   Common action sequences:
   - Form filling: [
       {"input_text": {"index": 1, "text": "username"}},
       {"input_text": {"index": 2, "text": "password"}},
       {"click_element": {"index": 3}}
     ]
   - Navigation and extraction: [
       {"go_to_url": {"url": "https://example.com"}},
       {"extract_page_content": {}}
     ]

3. ELEMENT INTERACTION:
   - Only use indexes that exist in the provided element list
   - Each element has a unique index number (e.g., "33[:]<button>")
   - Elements marked with "_[:]" are non-interactive (for context only)

4. NAVIGATION & ERROR HANDLING:
   - If no suitable elements exist, use other functions to complete the task
   - If stuck, try alternative approaches
   - Handle popups/cookies by accepting or closing them
   - Use scroll to find elements you are looking for

5. TASK COMPLETION:
   - If you think all the requirements of user's instruction have been completed and no further operation is required, output the **Done** action to terminate the operation process.
   - Don't hallucinate actions.
   - If the task requires specific information - make sure to include everything in the done function. This is what the user will see.
   - If you are running out of steps (current step), think about speeding it up, and ALWAYS use the done action as the last action.
   - Note that you must verify if you've truly fulfilled the user's request by examining the actual page content, not just by looking at the actions you output but also whether the action is executed successfully. Pay particular attention when errors occur during action execution.

6. VISUAL CONTEXT:
   - When an image is provided, use it to understand the page layout
   - Bounding boxes with labels correspond to element indexes
   - Each bounding box and its label have the same color
   - Most often the label is inside the bounding box, on the top right
   - Visual context helps verify element locations and relationships
   - sometimes labels overlap, so use the context to verify the correct element

7. Form filling:
   - If you fill an input field and your action sequence is interrupted, most often a list with suggestions poped up under the field and you need to first select the right element from the suggestion list.

8. ACTION SEQUENCING:
   - Actions are executed in the order they appear in the list 
   - Each action should logically follow from the previous one
   - If the page changes after an action, the sequence is interrupted and you get the new state.
   - If content only disappears the sequence continues.
   - Only provide the action sequence until you think the page will change.
   - Try to be efficient, e.g. fill forms at once, or chain actions where nothing changes on the page like saving, extracting, checkboxes...
   - only use multiple actions if it makes sense.
   - use maximum {max_actions_per_step} actions per sequence

Functions:
{default_action_description}

Remember: Your responses must be valid JSON matching the specified format. Each action in the sequence must be valid.
```

### 3.2 User Message Template

```
Current step: {step_number}/{max_steps}
Current date and time: {time_str}
1. Task: {task}. 
2. Hints(Optional): 
{add_infos}
3. Memory: 
{memory}
4. Current url: {url}
5. Available tabs:
{tabs}
6. Interactive elements:
{elements_text}

[If previous actions exist]:
**Previous Actions**
Previous step: {step_number-1}/{max_steps}
Previous action 1/{total_actions}: {action1_json}
Result of previous action 1/{total_actions}: {result1}
[If error]: Error of previous action 1/{total_actions}: {error1}
[Repeat for each action]

[If screenshot available]:
[IMAGE: Base64-encoded screenshot]
```

## 4. Available Actions

### 4.1 Navigation Actions

#### go_to_url
- **Description**: Navigate to a specified URL
- **Parameters**: 
  - `url`: String - Target URL to visit

#### open_tab
- **Description**: Open a new browser tab
- **Parameters**: 
  - `url`: Optional[String] - URL to open in new tab

#### switch_tab
- **Description**: Switch to a different browser tab
- **Parameters**: 
  - `index`: Integer - Tab index to switch to

### 4.2 Interaction Actions

#### click_element
- **Description**: Click on an element
- **Parameters**: 
  - `index`: Integer - Element index to click

#### input_text
- **Description**: Type text into an input field
- **Parameters**: 
  - `index`: Integer - Element index to type into
  - `text`: String - Text to input

#### send_keys
- **Description**: Send keyboard commands
- **Parameters**: 
  - `keys`: String - Keys to send (e.g., "Enter", "Tab")

#### scroll
- **Description**: Scroll the page
- **Parameters**: 
  - `direction`: String - "up", "down", or "to_element"
  - `amount`: Optional[Integer] - Pixels to scroll
  - `element_index`: Optional[Integer] - Element to scroll to

### 4.3 Content Extraction Actions

#### extract_page_content
- **Description**: Extract and process page content
- **Parameters**: 
  - `include_links`: Optional[Boolean] - Whether to include links in output

#### copy_to_clipboard
- **Description**: Copy text to clipboard
- **Parameters**: 
  - `text`: String - Text to copy

#### paste_from_clipboard
- **Description**: Paste text from clipboard
- **Parameters**: None

### 4.4 Control Actions

#### done
- **Description**: Signal task completion
- **Parameters**: 
  - `result`: Optional[String] - Final result information

## 5. Execution Flow

### 5.1 Task Execution

1. User submits a task via the `run_browser_agent` tool
2. Server initializes browser and agent components
3. Agent executes steps until completion:
   - Get current browser state
   - Format prompt with state information
   - Send prompt to model
   - Parse model response
   - Execute actions from response
   - Update state and history
   - Repeat until done or max steps reached
4. Server returns final result to user

### 5.2 Step Execution

1. Capture current browser state
2. Create user message with state information
3. Send message to model
4. Parse model's JSON response
5. Execute actions sequentially
6. Record results and update memory
7. Check for completion status

### 5.3 Error Handling

1. Track consecutive failures
2. Implement retry mechanisms
3. Provide error information in next prompt
4. Allow model to adjust strategy
5. Terminate after max failures threshold

## 6. Resource Management

### 6.1 Browser Resources

- Browser instance lifecycle management
- Context creation and cleanup
- Tab management
- Screenshot capture

### 6.2 Memory Management

- Important content extraction
- Progress tracking
- Task state persistence
- Error recording

## 7. Extension Points

### 7.1 Adding Custom Actions

1. Define action schema
2. Register action handler
3. Update prompt templates

### 7.2 Customizing Prompts

1. Extend system prompt
2. Modify user message format
3. Adjust response parsing

### 7.3 Enhancing Capabilities

1. Add new tools to MCP server
2. Extend browser functionality
3. Improve visual processing

## 8. Implementation Guidelines

Following best practices for implementing the Browser Automation MCP:

1. Build modular, extensible components
2. Implement robust error handling
3. Ensure secure browser automation
4. Optimize prompt design for consistent outputs
5. Support customization via environment variables
6. Implement telemetry for monitoring and debugging

## 9. Rust Implementation Considerations

When implementing this protocol in Rust with chromiumoxide, follow these specific guidelines:

### 9.1 Async Patterns

- Never use `async_trait` or `async fn` in traits
- Never return `Box<dyn Future>` or `Pin<Box<dyn Future>>` from client interfaces
- Provide synchronous interfaces with `.await()` called internally
- Hide async complexity behind `channel` and `task` `spawn`
- Return intuitive, domain-specific types (e.g., `AgentResponse`, `BrowserState`)

### 9.2 Error Handling

- Use `Result<T,E>` with custom errors
- No `unwrap()` except in tests
- Handle all Result/Option values explicitly
- Implement comprehensive error types with `thiserror`

### 9.3 Structure and Organization

- Keep each file under 300 lines
- Follow Rust official style (snake_case for variables/functions)
- Place tests in `tests/` directory
- Use `tracing` for logs with appropriate levels

This document provides a comprehensive schema definition for the Browser Automation MCP without implementation details, focusing on the protocol design and interaction patterns for AI models to effectively automate browser interactions.
