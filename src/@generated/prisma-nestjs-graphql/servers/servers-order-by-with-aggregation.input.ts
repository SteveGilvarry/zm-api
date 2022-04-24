import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { SortOrder } from '../prisma/sort-order.enum';
import { ServersCountOrderByAggregateInput } from './servers-count-order-by-aggregate.input';
import { ServersAvgOrderByAggregateInput } from './servers-avg-order-by-aggregate.input';
import { ServersMaxOrderByAggregateInput } from './servers-max-order-by-aggregate.input';
import { ServersMinOrderByAggregateInput } from './servers-min-order-by-aggregate.input';
import { ServersSumOrderByAggregateInput } from './servers-sum-order-by-aggregate.input';

@InputType()
export class ServersOrderByWithAggregationInput {

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

    @Field(() => ServersCountOrderByAggregateInput, {nullable:true})
    _count?: ServersCountOrderByAggregateInput;

    @Field(() => ServersAvgOrderByAggregateInput, {nullable:true})
    _avg?: ServersAvgOrderByAggregateInput;

    @Field(() => ServersMaxOrderByAggregateInput, {nullable:true})
    _max?: ServersMaxOrderByAggregateInput;

    @Field(() => ServersMinOrderByAggregateInput, {nullable:true})
    _min?: ServersMinOrderByAggregateInput;

    @Field(() => ServersSumOrderByAggregateInput, {nullable:true})
    _sum?: ServersSumOrderByAggregateInput;
}
