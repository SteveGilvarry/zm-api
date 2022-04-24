import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';

@InputType()
export class MonitorPresetsMinOrderByAggregateInput {

    @Field(() => SortOrder, {nullable:true})
    Id?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Name?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Type?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Device?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Channel?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Format?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Protocol?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Method?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Host?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Port?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Path?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    SubPath?: keyof typeof SortOrder;

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
    ControlId?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    ControlDevice?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    ControlAddress?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    DefaultRate?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    DefaultScale?: keyof typeof SortOrder;
}
