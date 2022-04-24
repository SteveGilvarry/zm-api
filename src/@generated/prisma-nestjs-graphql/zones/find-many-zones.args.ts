import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ZonesWhereInput } from './zones-where.input';
import { ZonesOrderByWithRelationInput } from './zones-order-by-with-relation.input';
import { ZonesWhereUniqueInput } from './zones-where-unique.input';
import { Int } from '@nestjs/graphql';
import { ZonesScalarFieldEnum } from './zones-scalar-field.enum';

@ArgsType()
export class FindManyZonesArgs {

    @Field(() => ZonesWhereInput, {nullable:true})
    where?: ZonesWhereInput;

    @Field(() => [ZonesOrderByWithRelationInput], {nullable:true})
    orderBy?: Array<ZonesOrderByWithRelationInput>;

    @Field(() => ZonesWhereUniqueInput, {nullable:true})
    cursor?: ZonesWhereUniqueInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => [ZonesScalarFieldEnum], {nullable:true})
    distinct?: Array<keyof typeof ZonesScalarFieldEnum>;
}
