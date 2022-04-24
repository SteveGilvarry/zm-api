import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Servers_Status } from './servers-status.enum';

@InputType()
export class NestedEnumServers_StatusFilter {

    @Field(() => Servers_Status, {nullable:true})
    equals?: keyof typeof Servers_Status;

    @Field(() => [Servers_Status], {nullable:true})
    in?: Array<keyof typeof Servers_Status>;

    @Field(() => [Servers_Status], {nullable:true})
    notIn?: Array<keyof typeof Servers_Status>;

    @Field(() => NestedEnumServers_StatusFilter, {nullable:true})
    not?: NestedEnumServers_StatusFilter;
}
