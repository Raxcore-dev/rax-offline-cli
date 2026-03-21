---
name: Bug Report
description: File a bug report
labels: ["bug", "triage"]
body:
  - type: markdown
    attributes:
      value: |
        Thanks for taking the time to fill out this bug report!
  - type: input
    id: version
    attributes:
      label: Rax Version
      description: What version of Rax are you using?
      placeholder: "0.1.0"
    validations:
      required: true
  - type: dropdown
    id: os
    attributes:
      label: Operating System
      description: What OS are you using?
      options:
        - Ubuntu 22.04
        - Ubuntu 20.04
        - Debian 12
        - Debian 11
        - Other Linux
        - macOS
        - Windows (WSL)
    validations:
      required: true
  - type: textarea
    id: what-happened
    attributes:
      label: What happened?
      description: Also tell us what you expected to happen
      placeholder: Describe the bug...
    validations:
      required: true
  - type: textarea
    id: reproduce
    attributes:
      label: Steps to Reproduce
      description: How can we reproduce this issue?
      placeholder: |
        1. Run 'rax ...'
        2. Type '...'
        3. See error
    validations:
      required: true
  - type: textarea
    id: logs
    attributes:
      label: Relevant Output
      description: Please copy and paste any relevant error messages or output
      render: shell
