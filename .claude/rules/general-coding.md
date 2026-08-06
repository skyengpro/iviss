
**Scope:** all files (general workflow rules)

# General Coding Guidelines

1. **Verify Information**: Always verify information before presenting it. Do not make
   assumptions or speculate without clear evidence.
2. **File-by-File Changes**: Make changes file by file and give the user a chance to spot
   mistakes.
3. **No Apologies**: Never use apologies.
4. **No Understanding Feedback**: Avoid giving feedback about understanding in comments or
   documentation.
5. **No Whitespace Suggestions**: Don't suggest whitespace-only changes.
6. **No Summaries**: Don't summarize changes made.
7. **No Inventions**: Don't invent changes other than what's explicitly requested.
8. **No Unnecessary Confirmations**: Don't ask for confirmation of information already provided
   in context.
9. **Preserve the architecture and structure of the repository**: Don't remove unrelated code or functionality. Preserve the structure and architecture whitout notify user with a real reason.
10. **Single Chunk Edits**: Provide all edits to a given file in a single chunk, not as
    multiple sequential steps.
11. **No Implementation Checks**: Don't ask the user to verify implementations already visible
    in context.
12. **No Unnecessary Updates**: Don't suggest changes to files when no actual modification is
    needed.
13. **Use Explicit Variable Names**: Prefer descriptive, explicit names over short, ambiguous
    ones.
14. **Follow Consistent Coding Style**: Match the existing style in the project.
15. **Prioritize Performance**: Consider and prioritize performance where applicable.
16. **Security-First Approach**: Always consider security implications when modifying or
    suggesting code.
17. **Test Coverage**: Suggest or include appropriate unit tests for new or modified code.
18. **Error Handling**: Implement robust error handling and logging where necessary.
19. **Modular Design**: Encourage modular design for maintainability and reusability.
20. **Version Compatibility**: Ensure changes are compatible with the project's specified
    language/framework versions.
21. **Avoid Magic Numbers**: Replace hardcoded values with named constants.
22. **Consider Edge Cases**: Always consider and handle potential edge cases.
23. **Use Assertions**: Include assertions where useful to validate assumptions and catch
    errors early.
24. **Code Comments**: Add comments only where the code genuinely needs them — avoid verbosity.
