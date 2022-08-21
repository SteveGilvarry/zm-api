import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ServersWhereInput } from './servers-where.input';
import { Type } from 'class-transformer';
import { ServersOrderByWithRelationInput } from './servers-order-by-with-relation.input';
import { ServersWhereUniqueInput } from './servers-where-unique.input';
import { Int } from '@nestjs/graphql';
import { ServersScalarFieldEnum } from './servers-scalar-field.enum';

@ArgsType()
export class FindManyServersArgs {

    @Field(() => ServersWhereInput, {nullable:true})
    @Type(() => ServersWhereInput)
    where?: ServersWhereInput;

    @Field(() => [ServersOrderByWithRelationInput], {nullable:true})
    @Type(() => ServersOrderByWithRelationInput)
    orderBy?: Array<ServersOrderByWithRelationInput>;

    @Field(() => ServersWhereUniqueInput, {nullable:true})
    @Type(() => ServersWhereUniqueInput)
    cursor?: ServersWhereUniqueInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => [ServersScalarFieldEnum], {nullable:true})
    distinct?: Array<keyof typeof ServersScalarFieldEnum>;
}
