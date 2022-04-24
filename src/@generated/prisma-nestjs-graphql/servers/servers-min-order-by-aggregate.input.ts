import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';

@InputType()
export class ServersMinOrderByAggregateInput {

    @Field(() => SortOrder, {nullable:true})
    Id?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Protocol?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Hostname?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Port?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    PathToIndex?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    PathToZMS?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    PathToApi?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Name?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    State_Id?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    Status?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    CpuLoad?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    TotalMem?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    FreeMem?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    TotalSwap?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    FreeSwap?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    zmstats?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    zmaudit?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    zmtrigger?: keyof typeof SortOrder;

    @Field(() => SortOrder, {nullable:true})
    zmeventnotification?: keyof typeof SortOrder;
}
