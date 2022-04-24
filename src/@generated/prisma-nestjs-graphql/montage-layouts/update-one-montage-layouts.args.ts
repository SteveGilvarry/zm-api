import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MontageLayoutsUpdateInput } from './montage-layouts-update.input';
import { MontageLayoutsWhereUniqueInput } from './montage-layouts-where-unique.input';

@ArgsType()
export class UpdateOneMontageLayoutsArgs {

    @Field(() => MontageLayoutsUpdateInput, {nullable:false})
    data!: MontageLayoutsUpdateInput;

    @Field(() => MontageLayoutsWhereUniqueInput, {nullable:false})
    where!: MontageLayoutsWhereUniqueInput;
}
