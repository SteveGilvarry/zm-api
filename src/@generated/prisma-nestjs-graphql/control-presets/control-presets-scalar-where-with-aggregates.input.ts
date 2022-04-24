import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntWithAggregatesFilter } from '../prisma/int-with-aggregates-filter.input';
import { StringWithAggregatesFilter } from '../prisma/string-with-aggregates-filter.input';

@InputType()
export class ControlPresetsScalarWhereWithAggregatesInput {

    @Field(() => [ControlPresetsScalarWhereWithAggregatesInput], {nullable:true})
    AND?: Array<ControlPresetsScalarWhereWithAggregatesInput>;

    @Field(() => [ControlPresetsScalarWhereWithAggregatesInput], {nullable:true})
    OR?: Array<ControlPresetsScalarWhereWithAggregatesInput>;

    @Field(() => [ControlPresetsScalarWhereWithAggregatesInput], {nullable:true})
    NOT?: Array<ControlPresetsScalarWhereWithAggregatesInput>;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    MonitorId?: IntWithAggregatesFilter;

    @Field(() => IntWithAggregatesFilter, {nullable:true})
    Preset?: IntWithAggregatesFilter;

    @Field(() => StringWithAggregatesFilter, {nullable:true})
    Label?: StringWithAggregatesFilter;
}
