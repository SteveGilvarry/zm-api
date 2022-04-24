import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Servers_Status } from './servers-status.enum';
import { NestedEnumServers_StatusFilter } from './nested-enum-servers-status-filter.input';

@InputType()
export class EnumServers_StatusFilter {

    @Field(() => Servers_Status, {nullable:true})
    equals?: keyof typeof Servers_Status;

    @Field(() => [Servers_Status], {nullable:true})
    in?: Array<keyof typeof Servers_Status>;

    @Field(() => [Servers_Status], {nullable:true})
    notIn?: Array<keyof typeof Servers_Status>;

    @Field(() => NestedEnumServers_StatusFilter, {nullable:true})
    not?: NestedEnumServers_StatusFilter;
}
