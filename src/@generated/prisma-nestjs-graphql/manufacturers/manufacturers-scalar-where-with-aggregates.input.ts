import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntWithAggregatesFilter } from '../prisma/int-with-aggregates-filter.input';
import { StringWithAggregatesFilter } from '../prisma/string-with-aggregates-filter.input';

@InputType()
export class ManufacturersScalarWhereWithAggregatesInput {

    @Field(() => [ManufacturersScalarWhereWithAggregatesInput], {nullable:true})
    AND?: Array<ManufacturersScalarWhereWithAggregatesInput>;

    @Field(() => [ManufacturersScalarWhereWithAggregatesInput], {nullable:true})
    OR?: Array<ManufacturersScalarWhereWithAggregatesInput>;

    @Field(() => [ManufacturersScalarWhereWithAggregatesInput], {nullable:true})
    NOT?: Array<ManufacturersScalarWhereWithAggregatesInput>;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    Id?: IntWithAggregatesFilter;

    @Field(() => StringWithAggregatesFilter, {nullable:true})
    Name?: StringWithAggregatesFilter;
}
