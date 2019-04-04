#!/usr/bin/env -S perl -i -p
# next if /^ *;/; s/\b(?<!-)if\b(?!-)/if-raw whsp1/g;
# next if /^ *;/; s/\b(?<!-)then\b(?!-)/then-raw whsp1/g;
# next if /^ *;/; s/\b(?<!-)else\b(?!-)/else-raw whsp1/g;
# next if /^ *;/; s/\b(?<!-)let\b(?!-)/let-raw whsp1/g;
# next if /^ *;/; s/\b(?<!-)in\b(?!-)/in-raw whsp1/g;
# next if /^ *;/; s/\b(?<!-)as\b(?!-)/as-raw whsp1/g;
# next if /^ *;/; s/\b(?<!-)using\b(?!-)/using-raw whsp1/g;
# next if /^ *;/; s/\b(?<!-)merge\b(?!-)/merge-raw whsp1/g;
# next if /^ *;/; s/\b(?<!-)Some\b(?!-)/Some-raw whsp1/g;

# next if /^ *;/; s/\b(?<!-)Optional\b(?!-)/Optional whsp/g;
# next if /^ *;/; s/\b(?<!-)Text\b(?!-)/Text whsp/g;
# next if /^ *;/; s/\b(?<!-)List\b(?!-)/List whsp/g;

# next if /^ *;/; s/\b(?<!-)or\b(?!-)/"||" whsp/g;
# next if /^ *;/; s/\b(?<!-)plus\b(?!-)/"+" whsp1/g;
# next if /^ *;/; s/\b(?<!-)text-append\b(?!-)/"++" whsp/g;
# next if /^ *;/; s/\b(?<!-)list-append\b(?!-)/"#" whsp/g;
# next if /^ *;/; s/\b(?<!-)and\b(?!-)/"&&" whsp/g;
# next if /^ *;/; s/\b(?<!-)times\b(?!-)/"*" whsp/g;
# next if /^ *;/; s/\b(?<!-)double-equal\b(?!-)/"==" whsp/g;
# next if /^ *;/; s/\b(?<!-)not-equal\b(?!-)/"!=" whsp/g;
# next if /^ *;/; s/\b(?<!-)equal\b(?!-)/"=" whsp/g;
# next if /^ *;/; s/\b(?<!-)dot\b(?!-)/"." whsp/g;
# next if /^ *;/; s/\b(?<!-)bar\b(?!-)/"|" whsp/g;
# next if /^ *;/; s/\b(?<!-)comma\b(?!-)/"," whsp/g;
# next if /^ *;/; s/\b(?<!-)at\b(?!-)/"@" whsp/g;
# next if /^ *;/; s/\b(?<!-)open-parens\b(?!-)/"(" whsp/g;
# next if /^ *;/; s/\b(?<!-)close-parens\b(?!-)/")" whsp/g;
# next if /^ *;/; s/\b(?<!-)open-brace\b(?!-)/"{" whsp/g;
# next if /^ *;/; s/\b(?<!-)close-brace\b(?!-)/"}" whsp/g;
# next if /^ *;/; s/\b(?<!-)open-bracket\b(?!-)/"[" whsp/g;
# next if /^ *;/; s/\b(?<!-)close-bracket\b(?!-)/"]" whsp/g;
# next if /^ *;/; s/\b(?<!-)open-angle\b(?!-)/"<" whsp/g;
# next if /^ *;/; s/\b(?<!-)close-angle\b(?!-)/">" whsp/g;
# next if /^ *;/; s/\b(?<!-)colon\b(?!-)/":" whsp1/g;
# next if /^ *;/; s/\b(?<!-)import-alt\b(?!-)/"?" whsp1/g;

# next if /^ *;/; s/\b(?<!-)combine-types\b(?!-)/combine-types whsp/g;
# next if /^ *;/; s/\b(?<!-)combine\b(?!-)/combine whsp/g;
# next if /^ *;/; s/\b(?<!-)prefer\b(?!-)/prefer whsp/g;
# next if /^ *;/; s/\b(?<!-)lambda\b(?!-)/lambda whsp/g;
# next if /^ *;/; s/\b(?<!-)forall\b(?!-)/forall whsp/g;
# next if /^ *;/; s/\b(?<!-)arrow\b(?!-)/arrow whsp/g;

# next if /^ *;/; s/\b(?<!-)label\b(?!-)/label whsp/g;
# next if /^ *;/; s/\b(?<!-)identifier\b(?!-)/identifier whsp/g;
# next if /^ *;/; s/\b(?<!-)text-literal\b(?!-)/text-literal whsp/g;
# next if /^ *;/; s/\b(?<!-)double-literal\b(?!-)/double-literal whsp/g;
# next if /^ *;/; s/\b(?<!-)integer-literal\b(?!-)/integer-literal whsp/g;
# next if /^ *;/; s/\b(?<!-)natural-literal\b(?!-)/natural-literal whsp/g;

# next if /^ *;/; s/\b(?<!-)import-hashed\b(?!-)/import-hashed whsp/g;
# next if /^ *;/; s/\b(?<!-)import-type\b(?!-)/import-type whsp/g;
# next if /^ *;/; s/\b(?<!-)import\b(?!-)/import whsp/g;
# next if /^ *;/; s/\b(?<!-)local\b(?!-)/local whsp/g;
# next if /^ *;/; s/\b(?<!-)env\b(?!-)/env whsp/g;
# next if /^ *;/; s/\b(?<!-)http\b(?!-)/http whsp/g;
# next if /^ *;/; s/\b(?<!-)missing\b(?!-)/missing whsp/g;

# next if /^ *;/; s/\b(?<!-)labels\b(?!-)/labels whsp/g;
# next if /^ *;/; s/\b(?<!-)record-type-or-literal\b(?!-)/record-type-or-literal whsp/g;
# next if /^ *;/; s/\b(?<!-)non-empty-record-type-or-literal\b(?!-)/non-empty-record-type-or-literal whsp/g;
# next if /^ *;/; s/\b(?<!-)non-empty-record-type\b(?!-)/non-empty-record-type whsp/g;
# next if /^ *;/; s/\b(?<!-)non-empty-record-literal\b(?!-)/non-empty-record-literal whsp/g;
# next if /^ *;/; s/\b(?<!-)union-type-or-literal\b(?!-)/union-type-or-literal whsp/g;
# next if /^ *;/; s/\b(?<!-)non-empty-union-type-or-literal\b(?!-)/non-empty-union-type-or-literal whsp/g;
# next if /^ *;/; s/\b(?<!-)non-empty-list-literal\b(?!-)/non-empty-list-literal whsp/g;
#
# next if /^ *;/; s/\b(?<!-)expression\b(?!-)/expression whsp/g;
# next if /^ *;/; s/\b(?<!-)lambda-expression\b(?!-)/lambda-expression whsp/g;
# next if /^ *;/; s/\b(?<!-)ifthenelse-expression\b(?!-)/ifthenelse-expression whsp/g;
# next if /^ *;/; s/\b(?<!-)let-expression\b(?!-)/let-expression whsp/g;
# next if /^ *;/; s/\b(?<!-)forall-expression\b(?!-)/forall-expression whsp/g;
# next if /^ *;/; s/\b(?<!-)arrow-expression\b(?!-)/arrow-expression whsp/g;
# next if /^ *;/; s/\b(?<!-)merge-expression\b(?!-)/merge-expression whsp/g;
# next if /^ *;/; s/\b(?<!-)annotated-expression\b(?!-)/annotated-expression whsp/g;
# next if /^ *;/; s/\b(?<!-)empty-list-or-optional\b(?!-)/empty-list-or-optional whsp/g;
# next if /^ *;/; s/\b(?<!-)empty-collection\b(?!-)/empty-collection whsp/g;
# next if /^ *;/; s/\b(?<!-)non-empty-optional\b(?!-)/non-empty-optional whsp/g;
# next if /^ *;/; s/\b(?<!-)operator-expression\b(?!-)/operator-expression whsp/g;
# next if /^ *;/; s/\b(?<!-)import-alt-expression\b(?!-)/import-alt-expression whsp/g;
# next if /^ *;/; s/\b(?<!-)or-expression\b(?!-)/or-expression whsp/g;
# next if /^ *;/; s/\b(?<!-)plus-expression\b(?!-)/plus-expression whsp/g;
# next if /^ *;/; s/\b(?<!-)text-append-expression\b(?!-)/text-append-expression whsp/g;
# next if /^ *;/; s/\b(?<!-)list-append-expression\b(?!-)/list-append-expression whsp/g;
# next if /^ *;/; s/\b(?<!-)and-expression\b(?!-)/and-expression whsp/g;
# next if /^ *;/; s/\b(?<!-)combine-expression\b(?!-)/combine-expression whsp/g;
# next if /^ *;/; s/\b(?<!-)prefer-expression\b(?!-)/prefer-expression whsp/g;
# next if /^ *;/; s/\b(?<!-)combine-types-expression\b(?!-)/combine-types-expression whsp/g;
# next if /^ *;/; s/\b(?<!-)times-expression\b(?!-)/times-expression whsp/g;
# next if /^ *;/; s/\b(?<!-)equal-expression\b(?!-)/equal-expression whsp/g;
# next if /^ *;/; s/\b(?<!-)not-equal-expression\b(?!-)/not-equal-expression whsp/g;
# next if /^ *;/; s/\b(?<!-)application-expression\b(?!-)/application-expression whsp/g;
#
# next if /^ *;/; s/\b(?<!-)import-expression\b(?!-)/import-expression whsp/g;
# next if /^ *;/; s/\b(?<!-)selector-expression\b(?!-)/selector-expression whsp/g;
# next if /^ *;/; s/\b(?<!-)primitive-expression\b(?!-)/primitive-expression whsp/g;
# next if /^ *;/; s/\b(?<!-)complete-expression\b(?!-)/complete-expression whsp/g;

# next if /^ *;/; s/\b(?<!-)whitespace\b(?!-)/whsp/g;
# next if /^ *;/; s/\b(?<!-)nonempty-whitespace\b(?!-)/whsp1/g;