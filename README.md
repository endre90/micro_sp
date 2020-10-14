# micro_sp

## TODO
### high prioriry 
1.  [x] error handling, probably not done
2.  [x] publish complete state
3.  [ ] need eqrr, ref has to hold, be == to act if trans is not affecting it! (then maybe there is no need for act_ref in raar)
4.  [ ] does this mean that act, ref and act_ref (if needed) have to have the same domain (and same r#type)?
5.  [ ] testing
6.  [ ] dummy robot node to publish act
7.  [x] documentation
8.  [ ] proper readme
9.  [x] proper get planning result
10. [ ] benchmarks to models
11. [ ] improve runner 
12. [ ] gui 
13. [ ] ref, act, act_ref of include invariants? (handshaking vaiants, try both why not (Cat::RAAR, Cat::INVAR))
14. [ ] parsing pddls, look into some libs, planners
15. [ ] generate dummy driver from the model
16. [ ] fix command lifetime
17. [x] match published data to sink when fresh timeouts
18. [ ] compositional algorithm
19. [ ] when to call the planner
20. [ ] gather complete state instead of measured
21. [ ] don't replan after every state change
22. [ ] handle estimated
23. [ ] move model to models
24. [ ] make model launch choice argument
25. [x] check for measured in domain
26. [ ] flow graph to readme

### low priority
1. [ ] nondeterminism
2. [ ] multiple goals
3. [ ] raar transition has to copy the unchanging from guard to update (modeling QoL)
4. [ ] dummy in var domain default or eliminate
5. [ ] dockerize
6. [ ] maybe some quality of life for modeling
7. [ ] generate both raar and invar cats from a hl model
8. [ ] explore all paths and generate graph