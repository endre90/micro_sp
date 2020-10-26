# micro_sp


## dummy 
A dummy driver (inverse of a micro_sp node) can be launched providing the argument bla bla bla

<img src='https://g.gravizo.com/svg?
 digraph G {
   micro_sp [shape=box];
   dummy [shape=box];
   micro_sp -> ref_1;
   micro_sp -> ref_2;
   ref_1 -> dummy;
   ref_2 -> dummy;
   dummy -> act_1 -> micro_sp;
   dummy -> act_2 -> micro_sp;
   dummy -> act_ref_1 -> micro_sp;
   dummy -> act_ref_2 -> micro_sp;
 }
'/>

## TODO
### high prioriry 
1.  [x] error handling, probably not done
2.  [x] publish complete state
3.  [x] update command vars to match measured values on refresh (raar)
4.  [x] raar: act, ref and act_ref have to have the same domain and same r#type
5.  [ ] where is the state? who owns the state?
6.  [x] proper command line arguments
7.  [ ] need new "start/stop running" and "set goal" topics to communicate with the runner
8.  [ ] the "start/stop running" should actually set the reference values     ?       
9.  [x] have to add delay for dummy act             
10. [ ] add boolean variables
11. [x] add handshake kind
12. [ ] the algorithms should probably be solver agnostic
13. [x] move setup to parser
14. [ ] other command paradigm or are we done?
15. [ ] testing
16. [x] structure and modules
17. [ ] generate dummy driver from the model
18. [x] documentation, probably not done
19. [ ] dummy_value should be deeply integrated
20. [ ] proper readme
21. [x] proper get planning result
22. [ ] include readmes to benches to describe added constraints
23. [ ] benchmarks blocksworld:
    1. [x] micro_sp inc enumerated boolean
    2. [ ] micro_sp inc pure boolean
    3. [ ] micro_sp inc pure enumeration
    4. [ ] micro_sp seq enumerated boolean
    5. [ ] micro_sp seq pure boolean
    6. [ ] micro_sp seq pure enumeration
    7. [ ] incplan
24. [ ] benchmarks barman
25. [ ] benchmarks childsnack
26. [ ] benchmarks gripper
27. [ ] benchmarks hiking
28. [ ] study conversion from a pddl model to a real world "runneble" model
29. [ ] enable other solvers beside z3
30. [ ] make a more general parser for boilerplate domain ? maybe not
31. [ ] clean the warnings
32. [ ] improve runner and don't replan after every state change
33. [ ] gui needed asap
34. [ ] show current plan in gui
35. [x] look into parsing pddls: conclusion: won't work
36. [ ] fix command lifetime
37. [x] match published data to sink when fresh timeouts
38. [ ] compositional algorithm
39. [ ] when to call the planner
40. [ ] handle estimated
41. [x] move model to models
42. [x] check if measured value is in domain
43. [ ] flow graph to readme
44. [ ] maybe try to write a cdcl solver?
45. [ ] some theories? hmm...

### low priority
1. [ ] improve Qol
2. [ ] solver agnostic, yes, but...
3. [ ] could micro_sp also be planner agnostic? try with dfs/bfs
4. [ ] notion of costs, add optimization support?
5. [ ] nondeterminism
6. [ ] multiple goals
7. [ ] dummy in var domain default or eliminate
8. [ ] dockerize
9. [ ] maybe some quality of life for modeling
10. [ ] generate both raar and invar paradigms from a hl model?
11. [ ] explore all paths and generate graph