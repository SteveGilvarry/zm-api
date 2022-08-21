import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MontageLayoutsUpdateInput } from './montage-layouts-update.input';
import { Type } from 'class-transformer';
import { MontageLayoutsWhereUniqueInput } from './montage-layouts-where-unique.input';

@ArgsType()
export class UpdateOneMontageLayoutsArgs {

    @Field(() => MontageLayoutsUpdateInput, {nullable:false})
    @Type(() => MontageLayoutsUpdateInput)
    data!: MontageLayoutsUpdateInput;

    @Field(() => MontageLayoutsWhereUniqueInput, {nullable:false})
    @Type(() => MontageLayoutsWhereUniqueInput)
    where!: MontageLayoutsWhereUniqueInput;
}
