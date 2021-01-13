:objects child1 child2 child3 child4 child5 child6 child7 child8 child9 child10 - child
:objects bread1 bread2 bread3 bread4 bread5 bread6 bread7 bread8 bread9 bread10 - bread-portion
:objects content1 content2 content3 content4 content5 content6 content7 content8 content9 content10 - content-portion
:objects tray1 tray2 tray3 - tray
:objects table1 table2 table3 kitchen - place
:objects sandw1 sandw2 sandw3 sandw4 sandw5 sandw6 sandw7 sandw8 sandw9 sandw10 sandw11 sandw12 sandw13 - sandwich

:init at tray1 kitchen
:init at tray2 kitchen
:init at tray3 kitchen
:init at_kitchen_bread bread1
:init at_kitchen_bread bread2
:init at_kitchen_bread bread3
:init at_kitchen_bread bread4
:init at_kitchen_bread bread5
:init at_kitchen_bread bread6
:init at_kitchen_bread bread7
:init at_kitchen_bread bread8
:init at_kitchen_bread bread9
:init at_kitchen_bread bread10
:init at_kitchen_content content1
:init at_kitchen_content content2
:init at_kitchen_content content3
:init at_kitchen_content content4
:init at_kitchen_content content5
:init at_kitchen_content content6
:init at_kitchen_content content7
:init at_kitchen_content content8
:init at_kitchen_content content9
:init at_kitchen_content content10
:init no_gluten_bread bread2
:init no_gluten_bread bread9
:init no_gluten_bread bread4
:init no_gluten_bread bread8
:init no_gluten_content content2
:init no_gluten_content content8
:init no_gluten_content content4
:init no_gluten_content content1
:init allergic_gluten child1
:init allergic_gluten child10
:init allergic_gluten child3
:init allergic_gluten child4
:init not_allergic_gluten child2
:init not_allergic_gluten child5
:init not_allergic_gluten child6
:init not_allergic_gluten child7
:init not_allergic_gluten child8
:init not_allergic_gluten child9
:init waiting child1 table2
:init waiting child2 table1
:init waiting child3 table1
:init waiting child4 table2
:init waiting child5 table3
:init waiting child6 table3
:init waiting child7 table3
:init waiting child8 table2
:init waiting child9 table1
:init waiting child10 table3
:init notexist sandw1
:init notexist sandw2
:init notexist sandw3
:init notexist sandw4
:init notexist sandw5
:init notexist sandw6
:init notexist sandw7
:init notexist sandw8
:init notexist sandw9
:init notexist sandw10
:init notexist sandw11
:init notexist sandw12
:init notexist sandw13

:goal served child1
:goal served child2
:goal served child3
:goal served child4
:goal served child5
:goal served child6
:goal served child7
:goal served child8
:goal served child9
:goal served child10