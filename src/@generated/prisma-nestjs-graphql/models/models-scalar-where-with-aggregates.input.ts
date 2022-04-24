import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntWithAggregatesFilter } from '../prisma/int-with-aggregates-filter.input';
import { StringWithAggregatesFilter } from '../prisma/string-with-aggregates-filter.input';
import { IntNullableWithAggregatesFilter } from '../prisma/int-nullable-with-aggregates-filter.input';

@InputType()
export class ModelsScalarWhereWithAggregatesInput {

    @Field(() => [ModelsScalarWhereWithAggregatesInput], {nullable:true})
    AND?: Array<ModelsScalarWhereWithAggregatesInput>;

    @Field(() => [ModelsScalarWhereWithAggregatesInput], {nullable:true})
    OR?: Array<ModelsScalarWhereWithAggregatesInput>;

    @Field(() => [ModelsScalarWhereWithAggregatesInput], {nullable:true})
    NOT?: Array<ModelsScalarWhereWithAggregatesInput>;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    Id?: IntWithAggregatesFilter;

    @Field(() => StringWithAggregatesFilter, {nullable:true})
    Name?: StringWithAggregatesFilter;

    @Field(() => IntNullableWithAggregatesFilter, {nullable:true})
    ManufacturerId?: IntNullableWithAggregatesFilter;
}
