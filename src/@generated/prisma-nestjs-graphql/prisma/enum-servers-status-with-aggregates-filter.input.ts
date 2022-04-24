import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Servers_Status } from './servers-status.enum';
import { NestedEnumServers_StatusWithAggregatesFilter } from './nested-enum-servers-status-with-aggregates-filter.input';
import { NestedIntFilter } from './nested-int-filter.input';
import { NestedEnumServers_StatusFilter } from './nested-enum-servers-status-filter.input';

@InputType()
export class EnumServers_StatusWithAggregatesFilter {

    @Field(() => Servers_Status, {nullable:true})
    equals?: keyof typeof Servers_Status;

    @Field(() => [Servers_Status], {nullable:true})
    in?: Array<keyof typeof Servers_Status>;

    @Field(() => [Servers_Status], {nullable:true})
    notIn?: Array<keyof typeof Servers_Status>;

    @Field(() => NestedEnumServers_StatusWithAggregatesFilter, {nullable:true})
    not?: NestedEnumServers_StatusWithAggregatesFilter;

    @Field(() => NestedIntFilter, {nullable:true})
    _count?: NestedIntFilter;

    @Field(() => NestedEnumServers_StatusFilter, {nullable:true})
    _min?: NestedEnumServers_StatusFilter;

    @Field(() => NestedEnumServers_StatusFilter, {nullable:true})
    _max?: NestedEnumServers_StatusFilter;
}
