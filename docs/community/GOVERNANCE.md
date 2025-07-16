# Governance Model

📖 **[← Back to Documentation](../README.md)** | 🏠 **[← Main README](../../README.md)**

This document describes the governance model for the wasm-sandbox project. It outlines how decisions are made, who has authority to make them, and how community members can participate in the decision-making process.

## Project Mission

The mission of wasm-sandbox is to provide a secure, efficient, and user-friendly WebAssembly-based sandboxing solution for untrusted code execution. We aim to democratize access to secure sandboxing technology and enable developers to build safer applications that can execute untrusted code with confidence.

## Governance Structure

wasm-sandbox follows a meritocratic governance model with defined roles and responsibilities.

### Roles

#### Users

Users are the people who use wasm-sandbox in their projects. They are valued members of the community whose feedback drives the project forward.

**Rights and Responsibilities:**

- Use the software according to its license
- Report bugs and suggest features
- Participate in community discussions
- Help other users
- Provide feedback on releases

#### Contributors

Contributors are community members who contribute code, documentation, examples, or other improvements to the project.

**Rights and Responsibilities:**

- Submit pull requests
- Report and triage issues
- Improve documentation
- Participate in feature discussions
- Help with testing and quality assurance

#### Maintainers

Maintainers are experienced contributors who have shown dedication to the project and have been granted additional permissions to manage the project.

**Rights and Responsibilities:**

- Review and merge pull requests
- Manage issues and feature requests
- Guide the project's technical direction
- Release new versions
- Enforce the code of conduct
- Mentor new contributors

#### Technical Steering Committee (TSC)

The Technical Steering Committee consists of core maintainers who make strategic decisions about the project's direction.

**Rights and Responsibilities:**

- Set the technical vision and roadmap
- Make final decisions on significant technical disputes
- Approve major architectural changes
- Manage project resources
- Approve new maintainers
- Update governance documents

### Current Project Leadership

#### Technical Steering Committee

- [Name] - [GitHub Username]
- [Name] - [GitHub Username]
- [Name] - [GitHub Username]

#### Maintainers

- [Name] - [GitHub Username] - [Area of Focus]
- [Name] - [GitHub Username] - [Area of Focus]
- [Name] - [GitHub Username] - [Area of Focus]

## Decision-Making Process

### Day-to-Day Decisions

Most day-to-day decisions are made through a consensus-seeking process within the appropriate repository:

1. An issue or pull request is created to propose a change
2. Community members discuss the proposal
3. Maintainers review and provide feedback
4. When consensus is reached, maintainers approve and implement the change

For minor changes that align with the project's direction, a single maintainer may approve and merge.

### Significant Changes

For significant changes (architectural changes, new features, API changes), we use a more formal process:

1. **RFC (Request for Comments)**: A detailed proposal is created as an issue or in the RFC repository
2. **Discussion Period**: The community has at least two weeks to review and comment
3. **Refinement**: The proposal is updated based on feedback
4. **Decision**: The TSC makes a final decision based on:
   - Technical merit
   - Alignment with project goals
   - Community feedback
   - Resource constraints
   - Maintenance implications

### Voting

When consensus cannot be reached, formal voting may be used:

- TSC members vote: +1 (approve), 0 (abstain), -1 (reject)
- A simple majority of TSC members must vote
- A supermajority (2/3) of votes must be +1 for approval
- Any -1 votes must include specific concerns that could change the vote if addressed

### Conflict Resolution

If conflicts arise:

1. Parties attempt to resolve the issue directly
2. If unresolved, a maintainer mediates
3. If still unresolved, the TSC makes a final decision

## Becoming a Maintainer

Contributors who have demonstrated the following qualities may be nominated as maintainers:

- Sustained contributions over time (code, documentation, reviews, etc.)
- Technical expertise
- Alignment with project goals and values
- Good judgment and communication skills
- Community leadership

Process:

1. An existing maintainer nominates a contributor
2. The nominee confirms their interest
3. TSC members discuss and vote privately
4. With a supermajority approval, the contributor becomes a maintainer

## Project Resources

The following resources are managed by the project leadership:

- GitHub repositories under the project organization
- CI/CD infrastructure
- Documentation websites
- Communication channels (Discord, mailing lists, etc.)
- Domain names and trademarks
- Financial resources (if any)

## Amendments to Governance

This governance document may be amended through the following process:

1. A proposal to change governance is submitted as an issue
2. The community discusses for at least two weeks
3. The TSC votes on the proposal
4. With approval, the changes are merged and announced

## Code of Conduct

All participants in the wasm-sandbox community are expected to adhere to the [Code of Conduct](GUIDELINES.md#code-of-conduct). The TSC is responsible for enforcing the code of conduct and responding to violations.

## Working Groups

For specific initiatives, the TSC may establish Working Groups to focus on particular areas:

### Formation

1. TSC approves the creation of a Working Group
2. A charter is established defining:
   - Scope and objectives
   - Expected deliverables
   - Timeline
   - Resource requirements
   - Reporting requirements

### Leadership

Each Working Group has:

- A chair appointed by the TSC
- Members selected based on expertise and interest

### Current Working Groups

- [Working Group Name] - [Focus Area] - [Chair]
- [Working Group Name] - [Focus Area] - [Chair]

## Release Process

New releases are managed according to the following process:

1. Maintainers propose a new release with a changelog
2. The TSC approves the release plan
3. A release candidate is created and tested
4. Community feedback is gathered
5. Blocking issues are addressed
6. The final release is approved and published
7. The release is announced to the community

## Communication

The project uses the following official communication channels:

- GitHub Issues and Discussions: Technical discussions and planning
- Discord Server: Community chat and real-time communication
- Mailing List: Important announcements and discussions
- Community Calls: Regular video meetings for synchronous discussions
- Blog: Project news and technical articles
- Twitter: Public announcements and community engagement

## Security Response Team

The project maintains a Security Response Team responsible for handling vulnerability reports:

- [Name] - [GitHub Username]
- [Name] - [GitHub Username]

The security response process is documented in [SECURITY.md](../../SECURITY.md).

## Project Dependencies

The project maintains a list of approved third-party dependencies. Adding new dependencies requires:

1. A proposal documenting the need and alternatives considered
2. Security and license compatibility review
3. Maintainer approval

## Intellectual Property

### Contributor License Agreement

Contributors maintain copyright to their contributions but grant the project a license under the project's license terms. Large contributions may require a formal Contributor License Agreement.

### Trademarks

The wasm-sandbox name and logo are trademarks of the project. Usage guidelines are documented in [TRADEMARK.md](TRADEMARK.md).

## Financial Management

If the project receives financial support:

1. The TSC is responsible for budget allocation
2. Financial decisions require TSC approval
3. Regular financial reports are made available to the community
4. Funds are used for:
   - Infrastructure costs
   - Development grants
   - Community events
   - Documentation improvements

## Annual Review

The TSC conducts an annual review of the project, including:

- Progress toward goals
- Community health
- Technical debt assessment
- Roadmap adjustments
- Governance effectiveness

The results are shared with the community.

## Succession Planning

If a maintainer or TSC member becomes inactive:

1. After 3 months of inactivity, they are contacted
2. After 6 months, their status is reviewed
3. Inactive members may step down or move to emeritus status
4. Replacement members are nominated and approved

## Project Dissolution

In the unlikely event that the project must be dissolved:

1. The community is notified at least 3 months in advance
2. Resources are transferred to a suitable steward
3. All code remains available under its license
4. Documentation is archived

## Conclusion

This governance model is designed to support the long-term health and growth of the wasm-sandbox project. It balances the need for clear decision-making processes with community participation and transparency.

---

*This document was last updated on: July 13, 2025*
