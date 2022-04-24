import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';

@InputType()
export class MonitorPresetsAvgOrderByAggregateInput {

    @Field(() => SortOrder, {nullable:true})
    Id?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Format?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Width?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Height?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Palette?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    MaxFPS?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Controllable?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    DefaultRate?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    DefaultScale?: keyof typeof SortOrder;
}
