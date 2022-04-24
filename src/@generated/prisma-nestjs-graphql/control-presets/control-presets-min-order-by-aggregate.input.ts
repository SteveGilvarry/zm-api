import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';

@InputType()
export class ControlPresetsMinOrderByAggregateInput {

    @Field(() => SortOrder, {nullable:true})
    MonitorId?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Preset?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Label?: keyof typeof SortOrder;
}
