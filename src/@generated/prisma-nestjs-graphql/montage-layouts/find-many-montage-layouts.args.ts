import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MontageLayoutsWhereInput } from './montage-layouts-where.input';
import { MontageLayoutsOrderByWithRelationInput } from './montage-layouts-order-by-with-relation.input';
import { MontageLayoutsWhereUniqueInput } from './montage-layouts-where-unique.input';
import { Int } from '@nestjs/graphql';
import { MontageLayoutsScalarFieldEnum } from './montage-layouts-scalar-field.enum';

@ArgsType()
export class FindManyMontageLayoutsArgs {

    @Field(() => MontageLayoutsWhereInput, {nullable:true})
    where?: MontageLayoutsWhereInput;

    @Field(() => [MontageLayoutsOrderByWithRelationInput], {nullable:true})
    orderBy?: Array<MontageLayoutsOrderByWithRelationInput>;

    @Field(() => MontageLayoutsWhereUniqueInput, {nullable:true})
    cursor?: MontageLayoutsWhereUniqueInput;

    @Field(() => Int, {nullable:true})
    take?: number;

    @Field(() => Int, {nullable:true})
    skip?: number;

    @Field(() => [MontageLayoutsScalarFieldEnum], {nullable:true})
    distinct?: Array<keyof typeof MontageLayoutsScalarFieldEnum>;
}
