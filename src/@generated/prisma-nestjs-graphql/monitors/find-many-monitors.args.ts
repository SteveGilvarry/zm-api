import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MonitorsWhereInput } from './monitors-where.input';
import { MonitorsOrderByWithRelationInput } from './monitors-order-by-with-relation.input';
import { MonitorsWhereUniqueInput } from './monitors-where-unique.input';
import { Int } from '@nestjs/graphql';
import { MonitorsScalarFieldEnum } from './monitors-scalar-field.enum';

@ArgsType()
export class FindManyMonitorsArgs {

    @Field(() => MonitorsWhereInput, {nullable:true})
    where?: MonitorsWhereInput;

    @Field(() => [MonitorsOrderByWithRelationInput], {nullable:true})
    orderBy?: Array<MonitorsOrderByWithRelationInput>;

    @Field(() => MonitorsWhereUniqueInput, {nullable:true})
    cursor?: MonitorsWhereUniqueInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => [MonitorsScalarFieldEnum], {nullable:true})
    distinct?: Array<keyof typeof MonitorsScalarFieldEnum>;
}
