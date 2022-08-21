import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ManufacturersWhereInput } from './manufacturers-where.input';
import { Type } from 'class-transformer';
import { ManufacturersOrderByWithRelationInput } from './manufacturers-order-by-with-relation.input';
import { ManufacturersWhereUniqueInput } from './manufacturers-where-unique.input';
import { Int } from '@nestjs/graphql';
import { ManufacturersScalarFieldEnum } from './manufacturers-scalar-field.enum';

@ArgsType()
export class FindManyManufacturersArgs {

    @Field(() => ManufacturersWhereInput, {nullable:true})
    @Type(() => ManufacturersWhereInput)
    where?: ManufacturersWhereInput;

    @Field(() => [ManufacturersOrderByWithRelationInput], {nullable:true})
    orderBy?: Array<ManufacturersOrderByWithRelationInput>;

    @Field(() => ManufacturersWhereUniqueInput, {nullable:true})
    cursor?: ManufacturersWhereUniqueInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => [ManufacturersScalarFieldEnum], {nullable:true})
    distinct?: Array<keyof typeof ManufacturersScalarFieldEnum>;
}
